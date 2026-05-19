# Deprecated EIPs

The three drafts in this folder are **deprecated and superseded by a single master standard**: [`../eip-aacp.md`](../eip-aacp.md) — *Agent Autonomous Commerce Protocol (AACP)*.

| Deprecated draft | Content now lives in |
|---|---|
| `eip-agent-capability-metadata.md` | AACP §5.2 Capability Declaration |
| `eip-erc8183-evaluator-arbitrator-roles.md` | AACP §5.4 Role Registry · §5.6 Dispute & Arbitration |
| `eip-capital-entrusted-agent-vault.md` | AACP §5.8 Vault Extension |

### Why the change

The three drafts each positioned themselves as *extensions to ERC-8183 / ERC-8004*. That framing under-sold AACP: the protocol defines a complete agent commerce and verification category — it composes with ERC-8004 (identity) and ERC-8183 (escrow semantics) as primitives, rather than patching them. Merging into one master EIP aligns AACP with the positioning model used by ERC-4337 (Account Abstraction), ERC-6551 (Token-Bound Accounts), and ERC-7579 (Modular Accounts).

Files are preserved here for historical reference only. Implementations SHOULD target the master AACP EIP exclusively.
