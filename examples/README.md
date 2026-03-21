# Example Verification Programs

Example programs that Clients can attach to ERC-8183 jobs. These define the acceptance criteria — the Evaluator runs them inside a zkVM to generate a ZK proof of the result.

## Program Specification

- **Format:** Compiled RISC-V ELF binary (target: `riscv32im-succinct-zkvm-elf` for SP1)
- **Input:** Deliverable content via stdin
- **Output:** Evaluation details via stdout
- **Result:** Exit code 0 = pass, non-zero = fail
- **Env vars:** `ZKVM_JOB_DESCRIPTION`, `ZKVM_DELIVERABLE_HASH`

## Examples

| Example | Language | Description |
|---------|----------|-------------|
| `erc20-checker/` | Rust | Verifies an ERC-20 contract has required functions |
| `json-schema-validator/` | Rust | Validates JSON deliverable against a schema |
| `word-count-checker/` | Rust | Checks document meets minimum word count |

## Building

### SP1 (Succinct)

```bash
cd erc20-checker
cargo prove build
# Output: target/elf/riscv32im-succinct-zkvm-elf
```

### RISC Zero

```bash
cd erc20-checker
cargo risczero build
# Output: target/riscv-guest/riscv32im-risc0-zkvm-elf/release/
```

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on adding new examples.
