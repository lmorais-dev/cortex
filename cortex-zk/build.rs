use std::{collections::HashMap, env, path::PathBuf};

use risc0_build::{DockerOptionsBuilder, GuestOptionsBuilder, embed_methods_with_options};
use risc0_build_ethereum::generate_solidity_files;

// Paths where the generated Solidity files will be written.
const SOLIDITY_IMAGE_ID_PATH: &str = "../cortex-contracts/src/ImageID.sol";
const SOLIDITY_ELF_PATH: &str = "../cortex-contracts/tests/Elf.sol";

fn main() {
    // Builds can be made deterministic, and thereby reproducible, by using Docker to build the
    // guest. Check the RISC0_USE_DOCKER variable and use Docker to build the guest if set.
    println!("cargo:rerun-if-env-changed=RISC0_USE_DOCKER");
    println!("cargo:rerun-if-changed=build.rs");
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let mut builder = GuestOptionsBuilder::default();
    if env::var("RISC0_USE_DOCKER").is_ok() {
        let docker_options = DockerOptionsBuilder::default()
            .root_dir(manifest_dir.join("../"))
            .build()
            .unwrap();
        builder.use_docker(docker_options);
    }
    let guest_options = builder.build().unwrap();

    // Generate Rust source files for the methods crate.
    let guests = embed_methods_with_options(HashMap::from([("cortex-zk-iseven", guest_options)]));

    // Generate Solidity source files for use with Forge.
    let image_id_path = manifest_dir.join(SOLIDITY_IMAGE_ID_PATH);
    let elf_path = manifest_dir.join(SOLIDITY_ELF_PATH);

    println!("{}", image_id_path.display());
    println!("{}", elf_path.display());

    let solidity_opts = risc0_build_ethereum::Options::default()
        .with_image_id_sol_path(image_id_path)
        .with_elf_sol_path(elf_path);

    generate_solidity_files(guests.as_slice(), &solidity_opts).unwrap();
}
