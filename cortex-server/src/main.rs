use std::str::FromStr;
use alloy::network::EthereumWallet;
use alloy::primitives::{ChainId, TxHash};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use alloy::sol_types::SolValue;
use alloy_primitives::Address;
use color_eyre::eyre::bail;
use cortex_zk::CORTEX_ZK_ISEVEN_ELF;
use risc0_zkvm::sha::Digestible;
use risc0_zkvm::{ExecutorEnv, InnerReceipt, ProverOpts, VerifierContext, default_prover};

sol!(
    #[sol(rpc, all_derives)]
    "../cortex-contracts/src/IIsEven.sol"
);

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let (seal, journal) = generate_proof().await?;

    println!("Generated proof: {:?}", const_hex::encode(&seal));
    println!("Journal: {:?}", journal);
    
    let tx_hash = save_number(journal, seal).await?;
    println!("Saved number: {:?}", tx_hash);

    Ok(())
}

async fn generate_proof() -> color_eyre::Result<(Vec<u8>, u128)> {
    let program_input = 18726318923102_u128.abi_encode();

    let executor_environment = ExecutorEnv::builder()
        .write_slice(&program_input)
        .build()
        .map_err(|error| {
            eprintln!(
                "😭 Failed to initialize the Executor Environment: {}",
                error
            );
            color_eyre::eyre::eyre!("Failed to initialize the Executor Environment: {}", error)
        })?;

    let receipt = default_prover()
        .prove_with_ctx(
            executor_environment,
            &VerifierContext::default(),
            CORTEX_ZK_ISEVEN_ELF,
            &ProverOpts::groth16(),
        )
        .map_err(|error| {
            eprintln!("😭 Failed to generate the proof: {}", error);
            color_eyre::eyre::eyre!("Failed to generate the proof: {}", error)
        })?
        .receipt;

    let seal = encode_seal(&receipt)?;
    let journal = <u128>::abi_decode(&receipt.journal.bytes).map_err(|error| {
        eprintln!("😭 Failed to decode the journal: {}", error);
        color_eyre::eyre::eyre!("Failed to decode the journal: {}", error)
    })?;

    Ok((seal, journal))
}

fn encode_seal(receipt: &risc0_zkvm::Receipt) -> color_eyre::Result<Vec<u8>> {
    let seal = match receipt.inner.clone() {
        InnerReceipt::Fake(receipt) => {
            let seal = receipt.claim.digest().as_bytes().to_vec();
            let selector = &[0xFFu8; 4];
            // Create a new vector with the capacity to hold both selector and seal
            let mut selector_seal = Vec::with_capacity(selector.len() + seal.len());
            selector_seal.extend_from_slice(selector);
            selector_seal.extend_from_slice(&seal);
            selector_seal
        }
        InnerReceipt::Groth16(receipt) => {
            let selector = &receipt.verifier_parameters.as_bytes()[..4];
            // Create a new vector with the capacity to hold both selector and seal
            let mut selector_seal = Vec::with_capacity(selector.len() + receipt.seal.len());
            selector_seal.extend_from_slice(selector);
            selector_seal.extend_from_slice(receipt.seal.as_ref());
            selector_seal
        }
        _ => bail!("Unsupported receipt type"),
        // TODO(victor): Add set verifier seal here.
    };
    Ok(seal)
}

async fn save_number(journal: u128, seal: Vec<u8>) -> color_eyre::Result<TxHash> {
    let wallet_raw = std::env::var("ETH_WALLET_PRIVATE_KEY").expect("Wallet not set");
    let rpc_url = std::env::var("RPC_URL").expect("RPC URL not set");
    let contract_address = std::env::var("CONTRACT_ADDRESS").expect("Contract address not set");
    let contract_address = Address::from_str(&contract_address).expect("Contract address in wrong format");

    let wallet = EthereumWallet::new(PrivateKeySigner::from_str(&wallet_raw).expect("Wallet in wrong format"));

    let fill_provider = ProviderBuilder::new().wallet(wallet)
        .with_chain_id(ChainId::from(43113_u64))
        .connect_http(rpc_url.parse()?);

    let contract = IIsEven::new(contract_address.clone(), fill_provider);
    let pending_tx = contract.set(journal, seal.into()).send().await.map_err(|error| {
        eprintln!("😭 Failed to save the number: {}", error);
        color_eyre::eyre::eyre!("Failed to save the number: {}", error)
    })?
        .with_timeout(Some(std::time::Duration::from_secs(10)))
        .with_required_confirmations(3);

    let tx_receipt = pending_tx.get_receipt().await.map_err(|error| {
        eprintln!("😭 Failed to get the receipt: {}", error);
        color_eyre::eyre::eyre!("Failed to get the receipt: {}", error)
    })?;

    Ok(tx_receipt.transaction_hash)
}