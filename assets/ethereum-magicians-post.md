# Draft: Ethereum Magicians discussion post

Paste-ready draft for opening the AACP and Skill Registry EIPs for community feedback. Two related but separable posts are included; each can be submitted on its own thread and they can cross-link.

---

## Post 1 — AACP (Agent Autonomous Commerce Protocol)

**Category:** `ERC` / Final call for feedback before EIP number request
**Title:** `[ERC] Agent Autonomous Commerce Protocol (AACP) — a standalone standard for autonomous-agent commerce, skill-based verification, and dispute arbitration`

Hi all,

We're proposing **AACP**, a standalone ERC that specifies the complete commerce-and-verification loop for autonomous agents operating on-chain: five roles, a verification-phase-aware job state machine, skill-based verification strategies, commit-reveal arbitration, a staking pool with structured slashing, and an optional vault extension for capital-entrusted jobs.

**Draft:** [`eip-aacp.md`](https://github.com/<org>/aacp/blob/main/eips/eip-aacp.md)
**Companion:** [`eip-skill-registry.md`](https://github.com/<org>/aacp/blob/main/eips/eip-skill-registry.md) — a separate, protocol-neutral Skill Registry that AACP depends on but that other agent frameworks can also adopt.

### Why one standard instead of several

Earlier drafts existed as three separate "extensions to ERC-8183 / ERC-8004". That framing obscured the coupling between the pieces and sold the protocol short. AACP is a category — not a patch — and we're positioning it the way ERC-4337 (Account Abstraction) positions relative to ERC-20: a self-contained specification that *uses* the base standards as primitives.

### What AACP is

- **Five roles** — Client, Provider, Evaluator, Arbitrator, Skill Author. Each has on-chain registration, reputation, and structured slashing.
- **Verification Strategies** — `program` (zkVM-deterministic), `rubric` (TEE multi-LLM / human / blended), `hybrid` (mixed), and `skill` (Skill-Registry-backed). Every job pins exactly one strategy at `createJob`.
- **Skill binding** — when `strategy.kind == Skill`, a `skillSetHash` is pinned on-chain. Evaluators and Arbitrators **MUST** load the identical resolved Skill versions inside a TEE; Providers **MAY** opt in for a royalty + reputation boost. The asymmetry is deliberate — it preserves market openness for proprietary Provider methodologies while keeping the verification side cryptographically bound.
- **Arbitration** — VRF-selected arbitrator sets, commit-reveal voting, protocol-initiated disputes (treasury covers the deposit when monitoring detects anomalies).
- **Optional vault extension** — §5.8 adds capital-entrusted job semantics (StrategyVault + TransactionGuard + CEX integration with zkTLS-attested off-chain balances).

### What it deliberately does NOT do

- It does not require Provider execution in a TEE. Execution environment is the Provider's choice; the protocol judges the deliverable, not the process.
- It does not require Providers to load the Client's selected Skill. Skill is a verifier-side primitive; production-side adoption is opt-in.
- It does not re-specify ERC-8004 identity or ERC-8183 escrow; it composes them.

### Open questions for the community

1. **Evaluator matching policy.** We specify both Client-selected and protocol-matched (VRF) as valid; should one be strictly preferred for high-value jobs?
2. **Dispute deposit scaling.** Our default is `budget × 5%` with multiplicative adjustments for strategy type, Evaluator record, and score-to-threshold proximity. Is this too permissive for low-budget jobs, or is the 5 USDC floor enough?
3. **Slashing defaults.** Section §5.7.2 now specifies first-offense/repeat percentages per violation type. Are any of the defaults mis-calibrated against simulations you've run for similar agent economies?
4. **Capability declaration unenforceability.** We explicitly limit on-chain enforcement of capability claims to role registration + Skill loading. Is this the right trade-off, or should we add more (e.g., TEE attestation binding to declared `verificationLevel`)?
5. **Skill Registry coupling.** Should the Skill Registry live as a separate EIP (our current plan) or should its full spec be re-embedded inside AACP? We believe the separation is better for cross-protocol reuse, but interested in contrary views.

### Reference implementation status

Solidity interfaces are stable. A reference implementation of `IAACP_Job`, `IAACP_Roles`, `IAACP_SkillRegistry`, `IAACP_Dispute`, `IAACP_Staking`, `IAACP_StrategyVault` is underway and will be published alongside the final EIP.

Comments, red-team analysis, and cross-references to prior work are all welcome. Particularly interested in feedback from:
- Teams building ERC-8004 / ERC-8183 deployments
- Account abstraction implementers (ERC-4337)
- Oracle and attested-computation designers (Chainlink Functions, TEE-NET, Risc0 Bonsai)
- Dispute-resolution protocols (Kleros, Aragon Court)

Thanks — we'll incorporate feedback before requesting an EIP number.

---

## Post 2 — Agent Skill Registry

**Category:** `ERC` / Draft
**Title:** `[ERC] Agent Skill Registry — a protocol-neutral registry for packaged domain expertise consumed by AI agents`

Posting as a companion to the AACP thread above, but worth discussing on its own because the registry is usable by any agent framework or protocol — not only AACP.

**Draft:** [`eip-skill-registry.md`](https://github.com/<org>/aacp/blob/main/eips/eip-skill-registry.md)

### What it is

A content-addressed, versioned on-chain registry for **Skills** — signed bundles of methodology, rubrics, verification programs, tool allowlists, and reference corpora. A Skill is authored by a domain expert, hash-pinned via IPFS/Arweave, and earns per-use royalties whenever a consumer declares a load.

Think **npm for agent domain expertise**, with on-chain integrity and optional royalty settlement.

### Why standalone

The same `solidity-audit` Skill could be consumed by:
- An on-chain agent marketplace (e.g., AACP)
- An off-chain code-review tool (Claude Skills, Cursor, custom runners)
- A bug-bounty platform verifying submissions
- An insurance underwriter sizing coverage

Embedding the registry inside any single protocol forces fragmentation. A standalone registry — analogous to ERC-165 or ERC-6551 — is a thin, composable primitive many higher-level protocols can share.

### Key design choices worth debating

- **Manifest on IPFS + CID on-chain.** Manifests reach hundreds of KB to MB (reference corpora, zkVM binaries). On-chain storage would be prohibitive; content addressing keeps integrity.
- **SemVer with immutable prior versions.** Critical for reproducibility when disputes arise months after use.
- **Generic `SkillLoaded` event.** Uses `(context, purposeTag, attestation)` triple so different consumer protocols can plug in without the registry knowing their roles.
- **Royalty hook is optional.** Off-chain consumers emit load events for attribution without triggering any on-chain payment. On-chain consumers use the hook when they want automatic royalty routing.
- **Author stake + slashing.** Malicious Skills (collusion) or stale Skills (uncorrected precision drops) lose a configurable stake slice.

### Interoperability with existing standards

- Compatible with the open [agentskills.io](https://agentskills.io) manifest format (which Claude Skills and other frameworks use).
- Orthogonal to ERC-2981 (token-transfer royalty); Skills are not NFTs and royalty is per-load.
- Optional: authors MAY mint an ERC-721 to represent Skill authorship if they want transfer semantics.

### Open questions

1. **Author-level reputation.** Should the registry track author-level reputation (across all their Skills) in addition to per-Skill reputation, to discourage whitewashing?
2. **Namespace curation.** Should the registry impose a namespace policy (e.g., author-prefixed names, domain-prefixed), or remain flat and rely on `authorStake` to disincentivize squatting?
3. **Sealed-corpus encryption scheme.** Are TEE-public-key encryption + re-encryption on rotation sufficient, or should we specify a shared-secret mechanism for cross-TEE-vendor compatibility?
4. **Oracle centralization.** Shadow-test oracles are privileged; the EIP requires them but doesn't specify a decentralization scheme. Interested in pointers to relevant prior work.

Thanks — feedback and prior art references welcome.
