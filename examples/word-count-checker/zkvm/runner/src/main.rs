use aacp_word_count_checker::{journal::Journal, MAX_BYTES};
use sha2::{Digest, Sha256};
use sp1_sdk::{
    blocking::{LightProver, Prover},
    Elf, SP1Stdin,
};
use std::{
    fs::File,
    io::{self, Read},
    process::ExitCode,
};

fn hash(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn run() -> Result<bool, &'static str> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        return Err("USAGE: runner ELF EXPECTED_ELF_SHA256 EXPECTED_INPUT_SHA256 < document");
    }
    if args[2..]
        .iter()
        .any(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err("INVALID_EXPECTED_HASH");
    }
    let mut elf = Vec::new();
    File::open(&args[1])
        .map_err(|_| "ELF_READ_FAILED")?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut elf)
        .map_err(|_| "ELF_READ_FAILED")?;
    if elf.len() > 16 * 1024 * 1024 || hex(&hash(&elf)) != args[2].to_ascii_lowercase() {
        return Err("ELF_HASH_MISMATCH");
    }
    let mut input = Vec::new();
    io::stdin()
        .lock()
        .take((MAX_BYTES + 2) as u64)
        .read_to_end(&mut input)
        .map_err(|_| "INPUT_READ_FAILED")?;
    if input.len() > MAX_BYTES + 1 {
        return Err("INPUT_FRAME_TOO_LARGE");
    }
    let digest = hash(&input);
    if hex(&digest) != args[3].to_ascii_lowercase() {
        return Err("INPUT_HASH_MISMATCH");
    }

    // Explicit execution-only backend: no from_env(), network prover, wallet,
    // fallback native checker, mock proof, or paid proving service.
    let client = LightProver::new();
    let mut stdin = SP1Stdin::new();
    stdin.write_slice(&input);
    let (public_values, report) = client
        .execute(Elf::from(elf), stdin)
        .cycle_limit(200_000_000)
        .calculate_gas(true)
        .run()
        .map_err(|_| "ZKVM_EXECUTION_FAILED")?;
    let journal = Journal::decode(public_values.as_slice()).ok_or("INVALID_PUBLIC_VALUES")?;
    if journal.input_sha256 != digest {
        return Err("PUBLIC_INPUT_HASH_MISMATCH");
    }
    println!("{{\"backend\":\"sp1-6.10.0-light-execution\",\"proofGenerated\":false,\"accepted\":{},\"code\":{},\"words\":{},\"inputSha256\":\"{}\",\"elfSha256\":\"{}\",\"instructions\":{}}}", journal.accepted, journal.code, journal.words, hex(&digest), args[2].to_ascii_lowercase(), report.total_instruction_count());
    Ok(journal.accepted)
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(code) => {
            eprintln!("{code}");
            ExitCode::FAILURE
        }
    }
}
