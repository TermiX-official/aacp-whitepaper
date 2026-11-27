#![no_main]
sp1_zkvm::entrypoint!(main);

use aacp_word_count_checker::{evaluate, journal::Journal, MAX_BYTES};
use sha2::{Digest, Sha256};

pub fn main() {
    // One raw frame. The host caps it at MAX_BYTES + 1 before framing; SP1's
    // input-region allocator reads the frame before this guest length check.
    let input = sp1_zkvm::io::read_vec();
    assert!(
        input.len() <= MAX_BYTES + 1,
        "input frame exceeds runner limit"
    );
    let digest: [u8; 32] = Sha256::digest(&input).into();
    let journal = Journal::new(digest, evaluate(&input));
    sp1_zkvm::io::commit_slice(&journal.encode());
    // SP1 execution success is distinct from document acceptance. Rejections
    // are committed too; the host returns exit 1 when journal.accepted=false.
}
