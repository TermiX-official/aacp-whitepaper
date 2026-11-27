# Word count checker

A small, dependency-free Rust reference for the word-count example listed in
the parent README. It implements the contribution guide's stdin, stdout and
exit-code contract. The counting core is `no_std` and performs no allocation.

An optional [SP1 adapter](zkvm/README.md) compiles this core into a RISC-V
guest and commits a document-hash-bound public result. The native CLI and its
tests still have no external dependencies.

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

There are two explicit execution paths: the native reference CLI above and
the SP1 guest/runner under `zkvm/`. The latter executes the RISC-V program
with SP1's execution-only backend. It generates no cryptographic proof and
does not establish a TEE attestation, on-chain job, or payment. See the adapter
guide for input framing, public-output encoding, resource limits and program/
document hash binding. Do not execute the native binary as a fallback for a
missing zkVM.

`ZKVM_JOB_DESCRIPTION` and `ZKVM_DELIVERABLE_HASH`, mentioned in the parent
specification, are deliberately not treated as authenticated facts here.
The native CLI does not compute or validate a deliverable hash; the separate
SP1 runner requires explicit expected hashes and checks the committed input
hash. Pin the verifier binary and policy before accepting work; do not let the
provider choose the verifier after delivery.

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
