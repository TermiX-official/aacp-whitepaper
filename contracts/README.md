# AACP Contract Interfaces

Reference Solidity interfaces for AACP-specific contracts. These extend the existing ERC-8004 and ERC-8183 standards.

## Contracts

| Contract | Status | Description |
|----------|--------|-------------|
| `IAACPStaking.sol` | Planned | Stake management, slash execution, reputation-weighted requirements |
| `IAACPReputation.sol` | Planned | Reputation computation from job outcomes |
| `IAACPDispute.sol` | Planned | Dispute filing, VRF arbitrator selection, commit-reveal settlement |
| `IAACPTreasury.sol` | Planned | Fee collection, bonus distribution, governance |
| `IAACPHook.sol` | Planned | ERC-8183 Hook connecting staking to job lifecycle |
| `IGroth16Verifier.sol` | Planned | On-chain zkVM proof verification |
| `ITeeAttestationVerifier.sol` | Planned | On-chain TEE attestation verification (Automata DCAP) |

## Dependencies

- [ERC-8004](https://eips.ethereum.org/EIPS/eip-8004): Identity Registry + Reputation Registry
- [ERC-8183](https://eips.ethereum.org/EIPS/eip-8183): ACPCore (Job Escrow + Lifecycle)
- [EIP-196/197](https://eips.ethereum.org/EIPS/eip-196): BN254 pairing precompiles (for Groth16 verification)
- [Automata DCAP](https://docs.ata.network): TEE attestation verification
