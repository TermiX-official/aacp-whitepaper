# Word count checker

A small, dependency-free Rust reference for the word-count example listed in
the parent README. It implements the contribution guide's stdin, stdout and
exit-code contract. The counting core is `no_std` and performs no allocation.

## Acceptance policy

- Input is a UTF-8 document, at most 65,536 bytes.
- At least **20 words** are required. A word is a nonempty run separated by
  Rust's ASCII whitespace definition (space, tab, LF, CR and form feed).
- Punctuation stays inside words. Non-ASCII whitespace is not a separator.
  This deliberately simple policy is unsuitable for languages requiring
  linguistic segmentation.
- Policy constants are compiled into the program. Deliverable text and
  environment metadata cannot lower the threshold.
- Output is one JSON line containing only a verdict, fixed code and counts.
  Exit status is **0 for pass, 1 for rejection or I/O failure**.

## Run locally

Requires stable Rust and Cargo. No dependencies, wallet, API key or network
access are required for building and running the native example.

```sh
cd examples/word-count-checker
cargo test --locked --offline
cargo build --release --locked --offline
./target/release/aacp-word-count-checker < fixtures/pass.txt
# accepted: true, words: 20; exit 0
./target/release/aacp-word-count-checker < fixtures/fail.txt
# accepted: false, code: TOO_FEW_WORDS; exit 1
```

The CLI reads at most 65,537 bytes before rejecting oversized input. It does
not read files by path, fetch URLs, execute deliverable content, read wallet
configuration, or echo document contents. An upstream runner should also cap
input size and enforce wall-clock limits for a stalled input stream.

## Integration boundaries

This contribution is a **native reference and portable verification core**,
not a completed SP1/RISC Zero guest or a generated proof. No zkVM execution,
TEE attestation, on-chain job, or payment is claimed. A guest adapter must
provide its VM-specific input framing, bounded input handling, and committed
public outputs; the host must check the program identity and bind the actual
input to the job's expected deliverable hash. Do not execute the native binary
as a fallback for a missing zkVM.

`ZKVM_JOB_DESCRIPTION` and `ZKVM_DELIVERABLE_HASH`, mentioned in the parent
specification, are deliberately not treated as authenticated facts here.
The program does not compute or validate a deliverable hash. Pin the verifier
binary and policy before accepting work; do not let the provider choose the
verifier after delivery.

Meeting a word count does **not** prove accuracy, originality, relevance or
buyer acceptance. This example is a mechanical criterion suitable for a
larger verification strategy, not a full research-quality evaluation.

## Checks

Tests cover the 19/20-word boundary, whitespace and Unicode behavior, the byte
limit, invalid UTF-8, real CLI stdin/exit codes, and non-disclosure of input.

```sh
cargo fmt --check
cargo clippy --locked --offline --all-targets -- -D warnings
```
