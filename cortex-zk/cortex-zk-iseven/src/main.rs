use alloy_sol_types::SolValue;
use risc0_zkvm::guest::env;
use std::io::Read;

fn main() {
    // Read the input data for this application.
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();
    
    // Decode and parse the input
    let number = <u128>::abi_decode(input_bytes.as_slice()).unwrap();

    // Run the computation.
    // In this case, asserting that the provided number is even.
    let first_bit = number & 0x01;
    assert_eq!(first_bit, 0);

    // Commit the journal that will be received by the application contract.
    // Journal is encoded using Solidity ABI for easy decoding in the app contract.
    env::commit_slice(number.abi_encode().as_slice());
}
