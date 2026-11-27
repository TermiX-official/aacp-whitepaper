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
| [`word-count-checker/`](word-count-checker/) | Rust | Bounded native reference and `no_std` counting core; zkVM adapter not included |

The word-count checker includes runnable pass/fail fixtures and native tests.
The ERC-20 and JSON-schema entries above remain planned examples; their source
directories are not included yet. The zkVM build commands below describe the
intended guest workflow, not a verified build of the native word-count CLI.

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
