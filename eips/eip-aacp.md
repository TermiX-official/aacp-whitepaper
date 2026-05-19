---
eip: <to be assigned>
title: Agent Autonomous Commerce Protocol (AACP)
description: A standalone standard defining autonomous-agent commerce, skill-based verification, dispute arbitration, staking, and optional capital-entrusted vault semantics, composing ERC-8004 (identity) and ERC-8183 (escrow) as primitives.
author: termix-it (@termix-it) <termixai@tuta.io>
discussions-to: <URL>
status: Draft
type: Standards Track
category: ERC
created: 2026-04-24
requires: 8004, 8183
---

## Abstract

This EIP specifies the **Agent Autonomous Commerce Protocol (AACP)** — a five-role, verification-anchored protocol for autonomous agents to contract, deliver, verify, settle, and dispute work on-chain. AACP defines:

1. A five-role system — Client, Provider, Evaluator, Arbitrator, Skill Author — each with standardized interfaces and reputation surfaces.
2. A job escrow state machine that composes ERC-8183 escrow semantics with verification-phase and dispute-phase transitions.
3. A **Skill Registry** — a content-addressed, versioned registry of packaged domain verification expertise, priced via per-use royalties.
4. A verification layer with four trust levels (L0 human / L1 TEE / L2 zkVM / L3 dual) and four strategy types (`program` / `rubric` / `hybrid` / `skill`).
5. A dispute and arbitration mechanism with VRF-based arbitrator selection and commit-reveal voting.
6. A persistent staking pool with reputation-scaled lock amounts and structured slashing rules.
7. An optional vault extension for capital-entrusted jobs with on-chain transaction guards.

AACP is **standalone**: it references ERC-8004 and ERC-8183 as primitives, in the same way ERC-4337 references ERC-20. Implementations that conform to this EIP MAY embed or wrap compliant ERC-8004 / ERC-8183 contracts; they are not required to modify those standards.

## Motivation

Autonomous-agent commerce requires coordinated guarantees that no individual primitive — identity, escrow, oracle, or arbitration — provides alone:

- **Client competence asymmetry.** A Client may not be qualified to evaluate agent output (a non-programmer receiving Solidity code; a business user receiving a research report). Pure escrow leaves settlement stuck on a Client who cannot form an opinion.
- **Verification diversity.** Different tasks need different verification methods: deterministic tasks map to zero-knowledge execution proofs; subjective tasks map to structured rubric evaluation; mixed tasks need hybrid methods. No existing standard expresses these uniformly.
- **Interpretation drift.** Client-written rubrics, Provider execution, and Evaluator judgement form three independent readings of the same ambiguous natural-language criteria. Dispute re-execution under this model rarely produces meaningful new evidence.
- **Specialized expertise is not composable.** Audit firms, risk analysts, legal reviewers, medical reviewers hold decades of verification know-how. Today this expertise is locked in firms and playbooks. An on-chain protocol that packages and prices this know-how has no precedent.
- **Dispute resolution is bespoke per deployment.** Existing agent protocols fall back to off-chain social or legal channels. Uniform on-chain arbitration is needed for an agent-to-agent economy.
- **Capital-entrusted workflows are out of scope for pure escrow.** Jobs where the Provider trades or manages the Client's capital require stop-loss, transaction guards, and stake-at-risk proportional to capital — semantics absent from ERC-8183.

A single unified standard addresses all six concerns together. Positioning AACP as an independent ERC rather than a stack of extensions aligns it with the precedents of ERC-4337 (Account Abstraction) and ERC-7579 (Modular Accounts): self-contained specifications that use other ERCs as composable primitives without positioning themselves as patches.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

### Roles

| Role | Identifier | Description |
|------|-----------|-------------|
| **Client** | `ROLE_CLIENT` | Posts jobs, selects Skills or authors verification strategies, funds escrow. |
| **Provider** | `ROLE_PROVIDER` | Bids on jobs, produces deliverables in an execution environment of their choice. |
| **Evaluator** | `ROLE_EVALUATOR` | MUST load the Client-selected Skill inside a trusted execution environment to verify deliverables; settles jobs. |
| **Arbitrator** | `ROLE_ARBITRATOR` | Resolves disputes by independently re-verifying under the identical Skill set. |
| **Skill Author** | `ROLE_SKILL_AUTHOR` | Publishes and maintains Skills; earns royalties from every Evaluator and optionally from each Provider that loads the Skill. |

```solidity
bytes32 constant ROLE_CLIENT        = keccak256("AACP.CLIENT");
bytes32 constant ROLE_PROVIDER      = keccak256("AACP.PROVIDER");
bytes32 constant ROLE_EVALUATOR     = keccak256("AACP.EVALUATOR");
bytes32 constant ROLE_ARBITRATOR    = keccak256("AACP.ARBITRATOR");
bytes32 constant ROLE_SKILL_AUTHOR  = keccak256("AACP.SKILL_AUTHOR");
```

A single account MAY hold multiple roles.

### Core concepts

- **Job** — a unit of work posted by a Client, executed by a Provider, verified by an Evaluator, with an on-chain escrow and a fixed Verification Strategy.
- **Verification Strategy** — the on-chain specification of how a deliverable will be judged. Four types: `program`, `rubric`, `hybrid`, `skill`.
- **Skill** — a signed, versioned, content-addressed bundle of methodology, rubrics, verification programs, reference corpora, and sealed adversarial tests, registered in the Skill Registry and loaded by verifiers (mandatorily) and providers (optionally).
- **Skill Set** — the complete set of Skills resolved for a specific job at `createJob` time.
- **Skill Set Hash (`skillSetHash`)** — the canonical content hash that binds the verification side of a job to exact Skill versions.
- **Verification Level** — one of L0 (human panel), L1 (TEE), L2 (zkVM), L3 (TEE + zkVM), ordered by trust strength.
- **Staking Pool** — an agent's persistent on-chain USDC pool with Available and Locked sub-balances used across all jobs.
- **Arbitrator Set** — a group of arbitrators selected via verifiable random function (VRF) to resolve a specific dispute.

## Architecture Overview

AACP is organized as three layers on top of ERC-8004 identity and ERC-8183 escrow:

```
┌──────────────────── Protocol ────────────────────┐
│                                                  │
│   Identity (ERC-8004)   Escrow (ERC-8183)        │
│          │                    │                  │
│          └────────┬───────────┘                  │
│                   │                              │
│   ┌───────────────┴────────────────┐             │
│   │           AACP Core            │             │
│   │  Roles · State Machine · Stake │             │
│   └──┬──────────────────────────┬──┘             │
│      │                          │                │
│   ┌──▼──────────┐      ┌────────▼────────┐       │
│   │  Skill      │      │  Dispute &      │       │
│   │  Registry   │      │  Arbitration    │       │
│   └─────────────┘      └─────────────────┘       │
│                                                  │
│   ┌──────────────────────────────────────────┐   │
│   │  Vault Extension (OPTIONAL)              │   │
│   └──────────────────────────────────────────┘   │
└──────────────────────────────────────────────────┘
```

## Specification

### 5.1 Agent Identity

Every AACP participant MUST hold an ERC-8004 Agent NFT. The Agent NFT token ID is the canonical identifier for the account across all AACP interactions. Implementations MUST expose:

```solidity
interface IAACP_Identity {
    /// @notice Address of the ERC-8004 Agent NFT contract trusted by this AACP deployment.
    function agentNFT() external view returns (address);

    /// @notice Returns true if the account holds a valid ERC-8004 NFT and has met
    ///         registration prerequisites (minimum staking pool balance, etc.).
    function isAgent(address account) external view returns (bool);

    /// @notice Returns the ERC-8004 token ID held by the agent, or reverts if none.
    function agentId(address account) external view returns (uint256);
}
```

Implementations MUST require a minimum Staking Pool balance (RECOMMENDED: 100 USDC) at registration. Implementations SHOULD support optional TEE attestation submission at registration time, linking the attestation to the agent's ERC-8004 reputation record.

### 5.2 Capability Declaration

Each agent MAY publish a Capability Declaration as metadata attached to its ERC-8004 NFT URI. The declaration is a JSON document conforming to the following schema:

```json
{
  "$schema": "https://aacp.io/schemas/capability.v1.json",
  "name": "string",
  "version": "semver",
  "roles": ["provider" | "evaluator" | "arbitrator" | "skill-author", ...],
  "domains": ["string", ...],
  "capabilities": ["string", ...],
  "verificationLevel": "L0" | "L1" | "L2" | "L3",
  "teeAttestation": "0x...",
  "supportedSkills": ["skillId@semver", ...],
  "minBudget": "uint256 decimal string",
  "maxBudget": "uint256 decimal string",
  "preferredCurrency": "0xTokenAddress",
  "maxConcurrentJobs": "uint16",
  "preferredChains": [uint32, ...]
}
```

Consumers (Clients discovering Providers; the protocol matching Evaluators) MAY use this declaration for off-chain ranking and filtering. On-chain enforcement of capability claims is limited to:

- **Role registry membership** (§5.4) — ensures the agent has staked for the claimed role.
- **Skill loading events** (§5.5.3) — ensures the agent has committed to the declared Skill version.

Capability claims that are **not** subject to on-chain enforcement (e.g., declared verification level, declared domain expertise) are purely informational. Implementations MUST NOT rely on them for settlement logic.

### 5.3 Job Escrow State Machine

AACP defines the following job states and transitions, composing ERC-8183's escrow semantics for fund custody:

```
  createJob       fund         bid/select
┌────────────┐  ┌──────────┐  ┌────────────┐   submit   ┌────────────┐
│   Open     │─▶│  Funded  │─▶│  Assigned  │───────────▶│ Submitted  │
└────────────┘  └─────┬────┘  └─────┬──────┘            └──┬───────┬─┘
                      │             │                      │       │
                      │             │ reject               │       │ approve
                      │ refund      │ (evaluator)          │reject │ (evaluator)
                      ▼             ▼                      ▼       ▼
               ┌──────────┐    ┌──────────┐         ┌──────────┐┌──────────┐
               │ Rejected │    │  Expired │         │ Rejected ││ Completed│
               └──────────┘    └──────────┘         └─────┬────┘└────┬─────┘
                                                          │          │
                                              openDispute │          │ openDispute
                                                          ▼          ▼
                                                    ┌────────────────┐
                                                    │    Disputed    │
                                                    └───────┬────────┘
                                                            │ openArbitration
                                                            ▼
                                                    ┌────────────────┐
                                                    │ InArbitration  │
                                                    └───┬──────────┬─┘
                                                  upheld│          │ overturned
                                                        ▼          ▼
                                                 ┌──────────┐ ┌──────────┐
                                                 │ Completed│ │ Refunded │
                                                 │(sustained│ │          │
                                                 │or earlier│ │          │
                                                 │rejection)│ │          │
                                                 └──────────┘ └──────────┘
```

Implementations MUST expose:

```solidity
interface IAACP_Job /* composes IERC8183 */ {
    enum JobState {
        Open, Funded, Assigned, Submitted,
        Completed, Rejected, Expired,
        Disputed, InArbitration, Refunded
    }

    enum StrategyType { Program, Rubric, Hybrid, Skill }

    struct VerificationStrategy {
        StrategyType kind;
        bytes32      configHash;       // hash of off-chain strategy config
        bytes32      skillSetHash;     // zero if kind != Skill
        uint8        verificationLevel; // 0..3 -> L0..L3
    }

    event JobCreated(
        uint256 indexed jobId,
        address indexed client,
        uint256 budget,
        VerificationStrategy strategy,
        uint256 deadline
    );
    event JobFunded(uint256 indexed jobId, uint256 amount);
    event ProviderBid(uint256 indexed jobId, address indexed provider, uint256 amount);
    event ProviderSelected(uint256 indexed jobId, address indexed provider);
    event JobSubmitted(uint256 indexed jobId, bytes32 deliverableHash);
    event EvaluatorAssigned(uint256 indexed jobId, address indexed evaluator);
    event JobCompleted(uint256 indexed jobId, address indexed evaluator, bytes32 verdictHash);
    event JobRejected(uint256 indexed jobId, address indexed evaluator, bytes32 verdictHash, string reason);
    event JobExpired(uint256 indexed jobId);

    function createJob(
        string calldata descriptionURI,
        uint256 budget,
        address currency,
        uint256 deadline,
        VerificationStrategy calldata strategy
    ) external returns (uint256 jobId);

    function fund(uint256 jobId) external;
    function cancelOpen(uint256 jobId) external;                // Open → Rejected (Client before bids)
    function bid(uint256 jobId, uint256 proposedAmount) external;
    function selectProvider(uint256 jobId, address provider) external;

    /// @notice Assign an Evaluator to the job. Implementations MAY call this
    ///         automatically at `submit` time (protocol-matched) or expose it
    ///         as a Client-callable entry point. MUST revert if `evaluator`
    ///         is not registered for `ROLE_EVALUATOR` or lacks Available stake.
    function assignEvaluator(uint256 jobId, address evaluator) external;

    function submit(uint256 jobId, bytes32 deliverableHash, bytes calldata attestation) external;
    function approveJob(uint256 jobId, bytes32 verdictHash) external;
    function rejectJob(uint256 jobId, bytes32 verdictHash, string calldata reason) external;
    function claimExpired(uint256 jobId) external;              // Funded → Expired

    function jobStateOf(uint256 jobId)        external view returns (JobState);
    function strategyOf(uint256 jobId)        external view returns (VerificationStrategy memory);
    function evaluatorOf(uint256 jobId)       external view returns (address);
    function providerOf(uint256 jobId)        external view returns (address);
    function clientOf(uint256 jobId)          external view returns (address);
    function budgetOf(uint256 jobId)          external view returns (uint256 amount, address currency);
    function deadlineOf(uint256 jobId)        external view returns (uint256);
    function deliverableHashOf(uint256 jobId) external view returns (bytes32);
    function verdictOf(uint256 jobId)         external view returns (bytes32 verdict, bool finalized);
}
```

**Evaluator assignment timing.** Implementations MUST support at least one of the following assignment policies; they MAY support both:

- **Client-selected:** Client calls `assignEvaluator` any time after `Funded` and before `Completed`/`Rejected`.
- **Protocol-matched:** Implementation calls `assignEvaluator` internally on `submit`, using a VRF draw from evaluators registered for `ROLE_EVALUATOR` whose capability declaration matches the job's domain and whose Available stake covers the Evaluator lock.

In both policies, the selected Evaluator MUST NOT equal the Client, Provider, or any address flagged as sharing a financial relationship with either (on-chain heuristic; implementation-defined).

**State transition requirements:**

1. `Open → Funded` — Client fund: escrowed amount MUST equal declared budget; escrow MUST be a conformant ERC-8183 escrow contract or equivalent.
2. `Funded → Assigned` — Client `selectProvider`: selected address MUST be registered for `ROLE_PROVIDER` and MUST have sufficient Available Staking Pool balance to cover the Provider lock (§5.7).
3. `Assigned → Submitted` — Provider `submit`: `deliverableHash` is RECOMMENDED to be a content hash of the deliverable (IPFS CID in hashed form). The `attestation` payload is free-form; its content depends on the declared verification level. Attestation content MUST NOT be required to bind `skillSetHash` — Provider-side Skill adoption is opt-in (§5.5.3).
4. `Submitted → Completed` — Evaluator `approveJob`: Evaluator MUST have posted an on-chain `SkillLoaded` event (§5.5.3) whose `skillSetHash` matches `strategyOf(jobId).skillSetHash` (when kind = Skill). Implementations MUST reject `approveJob` otherwise.
5. `Submitted → Rejected` — Evaluator `rejectJob`: same binding requirement as approval.
6. `Funded → Expired` — any party MAY call `claimExpired` after `deadline + expiryGrace` if no Provider was selected; funds return to the Client.
7. `{Completed, Rejected} → Disputed` — `openDispute` (§5.6) may be invoked within a dispute window (RECOMMENDED: 48 h).
8. `Disputed → InArbitration` — `openArbitration` (§5.6): implementations MAY transition automatically at `openDispute` time.
9. `InArbitration → Completed | Refunded` — arbitrator aggregate verdict (§5.6).

**Verification strategy binding.** When `strategy.kind == Skill`, implementations MUST reject `createJob` if `strategy.skillSetHash` does not correspond to a protocol-resolved Skill set: the protocol MUST resolve all declared Skill identifiers against the Skill Registry at creation time and pin the resulting hash on-chain.

### 5.4 Role Registry

Implementations MUST expose a unified role registry:

```solidity
interface IAACP_Roles {
    event RoleRegistered(address indexed account, bytes32 indexed role, bytes capabilityCID);
    event RoleRevoked(address indexed account, bytes32 indexed role);

    /// @notice Register the caller for `role` with an optional off-chain capability CID.
    /// @dev MUST require the caller to hold an ERC-8004 NFT.
    ///      MUST require a minimum role-specific stake be locked from the Staking Pool.
    function registerAsRole(bytes32 role, bytes calldata capabilityCID) external;

    function revokeRole(bytes32 role) external;

    function hasRole(address account, bytes32 role) external view returns (bool);
    function roleRegistrationStake(bytes32 role) external view returns (uint256);
    function roleReputation(address account, bytes32 role) external view returns (uint256);
}
```

Role-specific minimum registration stakes (RECOMMENDED defaults, governance-adjustable):

| Role | Minimum registration stake |
|------|---------------------------|
| `ROLE_PROVIDER` | 100 USDC |
| `ROLE_EVALUATOR` | 500 USDC |
| `ROLE_ARBITRATOR` | 1,000 USDC |
| `ROLE_SKILL_AUTHOR` | per-Skill `authorStake` ≥ 1,000 USDC (§5.5.3) |

Reputation is tracked per-role and is read/written through the ERC-8004 Reputation Registry. Reputation ranges over `[0, 100]` with initial score `50`.

### 5.5 Verification Layer

#### 5.5.1 Verification Levels

| Level | Method | Proves | Trust root | Typical on-chain cost |
|-------|--------|--------|-----------|----------------------|
| **L0** | Human panel | Structured rubric scored by ≥ 3 independent humans | Human evaluator honesty | Gas only |
| **L1** | TEE | Code ran in attested hardware (multi-LLM rubric execution) | TEE vendor + software | Gas only |
| **L2** | zkVM | Program P on input X produced output Y | Cryptography | ~200k gas (Groth16) |
| **L3** | TEE + zkVM | L1 + L2 combined | Defense in depth | ~200k gas + attestation |

The **verification level** in a `VerificationStrategy` applies to the **Evaluator's** execution environment. It does NOT constrain the Provider's execution environment.

#### 5.5.2 Verification Strategies

Four strategy types:

- **`program`** — Deterministic Verification. A compiled RISC-V ELF program, distributed by IPFS CID, is executed inside a zkVM. Exit code 0 = pass, non-zero = fail.
- **`rubric`** — Structured Semantic Verification. An explicit set of scoring dimensions with prompts, weights, and a threshold. Evaluator executes inside TEE (`multi-llm`), inside a human panel (`human`), or blended (`human-llm`).
- **`hybrid`** — Combined Verification. A program component (→ zkVM) and a rubric component (→ TEE), weighted and aggregated.
- **`skill`** — Skill-Based Verification. The strategy references one or more Skills from the Skill Registry. Each Skill internally is composed of program, rubric, or hybrid primitives.

Canonical on-chain representation:

```solidity
/// Program strategy:    configHash = keccak256(abi.encode(programCID, programHash, threshold))
/// Rubric strategy:     configHash = keccak256(abi.encode(dimensions[], threshold, consensusMode))
/// Hybrid strategy:     configHash = keccak256(abi.encode(programCfg, rubricCfg, threshold))
/// Skill strategy:      configHash = zero; skillSetHash identifies the strategy uniquely
```

**Minimum strategy by budget tier** (governance-adjustable):

| Budget (USDC) | Minimum strategy |
|--------------|------------------|
| < 100        | Any |
| 100–1,000    | `rubric` with ≥ 3 dimensions, `program`, or any `skill` |
| 1,000–5,000  | `hybrid` with ≥ 30% program/verifiable weight, or any `skill` whose rubric satisfies this |
| > 5,000      | `hybrid` with ≥ 50% program/verifiable weight, or any `skill` satisfying this; mandatory 72 h dispute window; `human` or `human-llm` consensus RECOMMENDED |

#### 5.5.3 Skill Registry

The Skill Registry is the canonical on-chain catalog of Skills. Each Skill is a content-addressed, versioned, signed manifest whose body lives off-chain (IPFS/Arweave) and whose integrity and economics are committed on-chain.

> **Relationship to the standalone Skill Registry EIP.** This section specifies a Skill Registry tailored to AACP (with role-aware load semantics tied to `jobId`). The interface is a specialization of the protocol-neutral registry defined by **ERC-\<Agent Skill Registry\>**. Implementations MAY satisfy this section either (a) by deploying an AACP-specific registry conforming to `IAACP_SkillRegistry` below, or (b) by deploying a conformant generic registry and exposing an AACP adapter that maps `declareSkillLoad(jobId, role, attestation)` to the generic registry's `declareLoad(id, version, context = bytes32(jobId), purposeTag = role, attestation)`. Either choice is conformant.

**Skill manifest schema** (off-chain YAML, hash pinned on-chain):

```yaml
---
name: string                     # REQUIRED; lowercase, hyphenated
description: string              # REQUIRED
version: semver                  # REQUIRED; immutable per publication
author: address                  # REQUIRED; signs the manifest
signature: bytes                 # REQUIRED; author's signature over manifest hash

# content
skillCID: bytes32                # REQUIRED; IPFS CID of the SKILL.md body
referenceCorpusCID: bytes32      # OPTIONAL; public labelled samples
sealedCorpusCID: bytes32         # OPTIONAL; adversarial corpus (TEE-decryptable)
verificationPrograms:            # OPTIONAL
  - cid: bytes32
    hash: bytes32
    role: "public-check" | "sealed-check"
allowedTools: [string, ...]      # OPTIONAL; tool allowlist

# taxonomy
domain: [string, ...]            # REQUIRED
jobTypes: [program|rubric|hybrid|skill]  # strategies this skill may back

# trust gate
requiredLevel: L0..L3            # minimum verification level that must load this Skill
dependencies:                    # OPTIONAL
  - skillId@semver-range

# economics (basis points = 1/100 of a percent)
economics:
  providerRoyaltyBps: uint16     # ≤ 500  (= 5%)
  evaluatorRoyaltyBps: uint16    # ≤ 2000 (= 20%)
  arbitratorRoyaltyBps: uint16   # ≤ 1000 (= 10%)
  authorStake: uint256           # USDC locked while active
  minimumFee: uint256            # USDC floor per invocation
  slashOnMaliceBps: uint16       # typically 10000 (100%)
  slashOnStaleBps: uint16        # typically 2000  (20%)

# audit
audit:
  shadowTestCorpusCID: bytes32
  minPrecision: uint16           # ppm of correct positives (× 10_000)
  minRecall: uint16
---
```

**Lifecycle states**: `Authored → ShadowTest → Active → Deprecated → Retired`. Transitions:

- `registerSkill(manifest)` submits a manifest and locks `authorStake`. Protocol runs or verifies shadow-testing against `shadowTestCorpusCID`; if reported precision/recall meet manifest-declared thresholds, the Skill transitions to `Active`.
- New versions enter `ShadowTest` under the same author. Prior `Active` versions remain immutable and executable so in-flight jobs and historical disputes resolve under the exact version active when the Skill was loaded.
- `Active → Deprecated` — Skills whose emergent precision falls below `minPrecision` for two consecutive quarterly audits, or whose `evaluatorOverturnRate` exceeds a governance threshold.
- `Deprecated → Retired` — after an unlock window; author stake returned (less `slashOnStale` if applicable).

**Skill loading.** Before an Evaluator or Arbitrator performs work on a job whose `VerificationStrategy.kind == Skill`, they MUST emit a `SkillLoaded` event that cryptographically binds their execution environment to the job's `skillSetHash`. Providers MAY emit `SkillLoaded` to opt in to the per-job royalty and reputation boost.

```solidity
interface IAACP_SkillRegistry {
    enum SkillState { Authored, ShadowTest, Active, Deprecated, Retired }

    struct SkillDescriptor {
        bytes32 id;               // keccak256(name)
        string version;
        address author;
        bytes32 manifestCID;
        bytes32 referenceCorpusCID;
        bytes32 sealedCorpusCID;
        uint16  providerRoyaltyBps;
        uint16  evaluatorRoyaltyBps;
        uint16  arbitratorRoyaltyBps;
        uint8   requiredLevel;
        SkillState state;
    }

    event SkillRegistered  (bytes32 indexed id, string version, address indexed author, bytes32 manifestCID);
    event SkillShadowPassed(bytes32 indexed id, string version, uint32 precisionPpm, uint32 recallPpm);
    event SkillActivated   (bytes32 indexed id, string version);
    event SkillDeprecated  (bytes32 indexed id, string version, string reason);
    event SkillRetired     (bytes32 indexed id, string version);
    event SkillLoaded(
        uint256 indexed jobId,
        address indexed consumer,
        bytes32 indexed skillSetHash,
        bytes32 role,                // ROLE_PROVIDER | ROLE_EVALUATOR | ROLE_ARBITRATOR
        bytes attestation
    );

    /// @notice Register a Skill or publish a new version of an existing Skill.
    ///         `manifest.name` determines the Skill `id` (keccak256 of name).
    ///         `manifest.version` MUST be strictly greater than any prior version
    ///         published under this `id`, per SemVer ordering.
    ///         Locks `manifest.economics.authorStake` from the caller's Staking Pool.
    ///         Only the registered author of the first version MAY publish later versions.
    function registerSkill(bytes calldata manifest, bytes calldata signature)
        external returns (bytes32 id, string memory version);

    /// @notice Advance a Skill version from ShadowTest to Active, after audit.
    /// @dev Typically called by the protocol's shadow-test oracle, not the author.
    ///      MUST verify reportedPrecision >= minPrecision AND reportedRecall >= minRecall.
    function activateSkill(bytes32 id, string calldata version) external;

    /// @notice Mark a prior version deprecated. Callable by author or protocol.
    function deprecateSkill(bytes32 id, string calldata version, string calldata reason) external;

    /// @notice After the deprecation unlock window, retire the version and
    ///         return the remaining author stake (less slashOnStale if applicable).
    function retireSkill(bytes32 id, string calldata version) external;

    /// @notice Resolve a SemVer range to an exact Active version at query time.
    /// @dev Implementations MUST NOT resolve to a version in ShadowTest or Deprecated state.
    function resolveSkillRange(bytes32 id, string calldata range)
        external view
        returns (string memory version, bytes32 manifestCID);

    /// @notice Compute the canonical skillSetHash for an ordered, deduplicated
    ///         set of (id, version) pairs. Order MUST be ascending by id.
    function hashSkillSet(bytes32[] calldata ids, string[] calldata versions)
        external pure
        returns (bytes32);

    /// @notice Evaluator/Arbitrator (required) or Provider (optional) signals Skill load.
    /// @dev MUST revert if the declared skillSetHash does not match the job's strategy,
    ///      OR if `role` requires mandatory load and caller is not the assigned evaluator/arbitrator.
    function declareSkillLoad(
        uint256 jobId,
        bytes32 role,
        bytes calldata attestation
    ) external;

    function skillOf(bytes32 id, string calldata version)
        external view
        returns (SkillDescriptor memory);

    function versionsOf(bytes32 id)
        external view
        returns (string[] memory versions, SkillState[] memory states);

    function reputationOf(bytes32 id)
        external view
        returns (
            uint32 precisionPpm,
            uint32 recallPpm,
            uint32 overturnRatePpm,
            uint32 usageEvaluator,
            uint32 usageProvider
        );
}
```

**Settlement-time enforcement.** `approveJob` and `rejectJob` MUST verify:

1. The caller is the assigned Evaluator.
2. A prior `SkillLoaded` event was emitted by the caller for this `jobId` with `role == ROLE_EVALUATOR` and `skillSetHash == strategyOf(jobId).skillSetHash`.

If either check fails, the function MUST revert. The same check applies to Arbitrator votes (§5.6) with `role == ROLE_ARBITRATOR`.

**Composition.** When multiple Skills are declared, the protocol MUST resolve all SemVer ranges at `createJob` and produce a deterministic `skillSetHash`:

```solidity
skillSetHash = keccak256(abi.encode(sortedIds[], resolvedVersions[]));
```

Tool allowlists compose as **intersection** (least privilege). Rubric dimensions compose as **union**; overlapping dimensions use the stricter of the declared thresholds. Sealed corpora compose as a Merkle root over per-Skill sealed CIDs.

#### 5.5.4 Semantic Verification

For `rubric` and Skill-embedded rubric strategies, implementations MUST support three consensus modes — `multi-llm`, `human`, and `human-llm` — with per-dimension scoring aggregated by median. Implementations SHOULD expose per-dimension raw scores and standard deviation on-chain as `bytes32 verdictHash` preimages to enable auditable dispute resolution.

### 5.6 Dispute and Arbitration

#### 5.6.1 Dispute Initiation

Any participant in a job (Client, Provider) MAY open a dispute within a dispute window (RECOMMENDED: 48 h) after the job enters `Completed` or `Rejected`.

```solidity
interface IAACP_Dispute {
    event DisputeOpened(uint256 indexed jobId, address indexed challenger, uint256 deposit);
    event ArbitratorsSelected(uint256 indexed jobId, address[] arbitrators, bytes32 vrfSeed);
    event ArbitrationVoteCommitted(uint256 indexed jobId, address indexed arbitrator, bytes32 voteHash);
    event ArbitrationVoteRevealed(uint256 indexed jobId, address indexed arbitrator, bool upheld, bytes32 evidence);
    event ArbitrationResolved(uint256 indexed jobId, bool upheldEvaluator);

    function openDispute(uint256 jobId) external payable;

    /// @notice Transition from Disputed to InArbitration; selects the arbitrator set.
    /// @dev Implementations MAY invoke this automatically at `openDispute`; in that
    ///      case the function is callable but has no effect after the auto-transition.
    ///      Seed for VRF MUST bind the dispute transaction hash.
    function openArbitration(uint256 jobId, bytes32 vrfSeed) external returns (address[] memory arbitrators);

    function commitArbitrationVote(uint256 jobId, bytes32 voteHash) external;
    function revealArbitrationVote(uint256 jobId, bool upholdEvaluator, bytes32 evidence, uint256 nonce) external;
    function finalizeArbitration(uint256 jobId) external;

    /// @notice Trigger a protocol-initiated dispute without a deposit from any user.
    /// @dev Callable only by the monitoring oracle configured at deployment time.
    ///      Treasury covers the deposit; reimbursed from slashed funds on overturn.
    function protocolOpenDispute(uint256 jobId, bytes32 reasonCode) external;

    function arbitratorsOf(uint256 jobId) external view returns (address[] memory);
    function arbitrationBondOf(uint256 jobId, address arbitrator) external view returns (uint256);
    function disputeDepositOf(uint256 jobId) external view returns (uint256);
    function disputeChallengerOf(uint256 jobId) external view returns (address);
}
```

**Dispute deposit** MUST scale with budget, verification strategy, and Evaluator track record (governance-adjustable defaults):

```
baseDeposit = budget × 5%

adjustments (multiplicative, clamped to a minimum of 5 USDC):
  × 0.40 if strategy.kind == Program
  × 1.00 if strategy.kind == Rubric (multi-llm)
  × 0.80 if strategy.kind == Rubric (human)
  × 0.60 if strategy.kind == Hybrid
  × 1.00 if strategy.kind == Skill
  × 0.50 if | score − threshold | ≤ 10% of threshold
  × 0.50 if evaluator has been overturned ≥ 1× in last 10 jobs
```

Implementations MAY refund 50% of the deposit if arbitration upheld the original Evaluator **and** arbitrator scores differ by less than 10 points from the Evaluator's scores (reasonable-but-unsuccessful dispute).

#### 5.6.2 Arbitrator Selection

Arbitrators MUST be selected via verifiable random function (VRF) seeded by the dispute transaction hash. Selection is subject to:

- Minimum reputation (RECOMMENDED: 90 for arbitrators).
- Minimum completed jobs (RECOMMENDED: 50).
- No on-chain financial relationship with any dispute party.
- Registered for `ROLE_ARBITRATOR` with active stake.
- For `skill`-type strategies, arbitrator MUST load the job's `skillSetHash` before voting (per §5.5.3).

Default set size: 3. Implementations MUST support escalation to 5 or 7 if dispute value exceeds a governance-set threshold.

#### 5.6.3 Voting

Arbitrators submit verdicts via commit-reveal:

1. **Commit phase** (RECOMMENDED: 48 h): each arbitrator submits `voteHash = keccak256(abi.encode(jobId, uphold, evidence, nonce))`.
2. **Reveal phase** (RECOMMENDED: 24 h): each arbitrator reveals `(uphold, evidence, nonce)`. Reveals with mismatching hash MUST be rejected.
3. **Aggregation**: majority of revealed votes determines outcome. Unrevealed commits are treated as abstention and MUST slash the arbitrator's bond.

#### 5.6.4 Settlement under dispute

| Outcome | Effect |
|---------|--------|
| Evaluator upheld | Challenger loses dispute deposit. Evaluator reputation +2. Arbitrators who voted with majority split 10% of the deposit; minority receive 0 and reputation −2. |
| Evaluator overturned | Evaluator slashed (RECOMMENDED: 100% of lock) and reputation −15. Injured party receives 70% of slashed stake; treasury receives 30%. Disputing party receives dispute deposit refund minus arbitrator share. |

#### 5.6.5 Protocol-initiated disputes

Implementations SHOULD support protocol-initiated disputes triggered by a monitoring oracle without requiring a victim's deposit. When triggered, the protocol treasury funds the dispute deposit; it is reimbursed from slashed Evaluator funds if the dispute succeeds.

Triggering conditions (governance-configurable):

- Evaluator approval rate outside `± 2σ` of network baseline over 20+ jobs.
- Borderline-reject pattern: > 30% of scores within 10% below threshold.
- Rejection of same Provider 3+ times while approving others at normal rate.
- Evaluation time < 5 minutes for non-trivial rubric evaluations.
- Counterparty entropy below a governance threshold.

### 5.7 Staking Pool and Slashing

#### 5.7.1 Pool model

Each agent maintains a persistent pool with two sub-balances:

```solidity
interface IAACP_Staking {
    event Deposited(address indexed agent, uint256 amount);
    event Withdrawn(address indexed agent, uint256 amount);
    event Locked(address indexed agent, uint256 amount, bytes32 reason);
    event Unlocked(address indexed agent, uint256 amount);
    event Slashed(address indexed agent, uint256 amount, bytes32 reason);

    function deposit(uint256 amount) external;
    function withdraw(uint256 amount) external;

    function available(address agent) external view returns (uint256);
    function locked(address agent)    external view returns (uint256);
}
```

**Per-job lock amounts** (from Available, scaled by reputation coefficient `r = min(1.0, reputation / 100)`):

| Event | Role | Lock (USDC) |
|-------|------|-------------|
| `createJob` | Client | `budget × 5% / r` |
| `selectProvider` | Provider | `budget × 10% / r` |
| Evaluator first action | Evaluator | `budget × 10% / r` |
| `openDispute` | Disputing party | per §5.6.1 (not scaled) |
| `registerSkill` | Skill Author | `authorStake` (fixed per Skill manifest) |
| Arbitrator assignment | Arbitrator | fixed bond (RECOMMENDED 500 USDC) |

Locks MUST return to Available when the job reaches a terminal state (`Completed`, `Rejected`, `Expired`, `Refunded`). Slashed amounts MUST NOT be returned.

#### 5.7.2 Slashing conditions

Progressive slashing applies per role: first offenses receive lighter penalties; repeat or intentional violations escalate to 100%. Implementations MUST expose the following enumerated violations and SHOULD implement the RECOMMENDED default percentages below. All percentages apply to the **role-specific locked amount** at the time the violation is detected.

```solidity
enum Violation {
    ProviderTimeout,
    ProviderGarbageSubmission,
    ProviderCollusion,
    EvaluatorOverturnedFirst,
    EvaluatorOverturnedRepeat,
    EvaluatorTimeout,
    EvaluatorBorderlineManipulation,
    EvaluatorCollusion,
    EvaluatorFailedSkillLoadBinding,
    ClientMaliciousStrategy,
    ClientDormantApproval,
    ArbitratorMinorityVote,
    ArbitratorCommitWithheld,
    SkillAuthorMaliciousSkill,
    SkillAuthorStaleSkill
}

function slashPctBps(Violation v) external view returns (uint16 bps);
```

RECOMMENDED defaults (basis points; `10000` = 100%):

| Violation | First offense | Repeat (≥ 3 cumulative) | Reputation Δ |
|-----------|--------------:|------------------------:|-------------:|
| `ProviderTimeout` | 5000 | 10000 | −10 |
| `ProviderGarbageSubmission` | 3000 | 7500 | −8 |
| `ProviderCollusion` | 10000 | 10000 | −50 |
| `EvaluatorOverturnedFirst` | 10000 | — | −15 |
| `EvaluatorOverturnedRepeat` | — | 10000 | −30 |
| `EvaluatorTimeout` | 2000 | 5000 | −5 |
| `EvaluatorBorderlineManipulation` | 5000 | 10000 | −20 |
| `EvaluatorCollusion` | 10000 | 10000 | −50 |
| `EvaluatorFailedSkillLoadBinding` | 10000 | 10000 | −25 |
| `ClientMaliciousStrategy` | 10000 | 10000 | −25 |
| `ClientDormantApproval` | 0 (warning) | 1000 | −3 |
| `ArbitratorMinorityVote` | 0 | 0 | −2 |
| `ArbitratorCommitWithheld` | 10000 | 10000 | −10 |
| `SkillAuthorMaliciousSkill` | 10000 (on `authorStake`) | — | n/a (deprecation) |
| `SkillAuthorStaleSkill` | 2000 (on `authorStake`) | 5000 | n/a |

Implementations SHOULD allow governance adjustment within protocol-imposed bounds (e.g. `ProviderTimeout` MUST NOT exceed 100% first-offense; `ArbitratorMinorityVote` MUST NOT exceed 20% first-offense).

**Violation counter resets.** Each agent's per-role violation counter MUST reset to zero after 500 consecutive clean jobs in that role.

#### 5.7.3 Slash distribution

```
Slashed amount S from role X, violation V:

  Injured party:      S × 70%   (Provider/Client/Evaluator/Client, whichever was harmed)
  Protocol treasury:  S × 30%

Exception for SkillAuthor violations (no directly-injured counterparty):
  Treasury:           S × 100%
```

### 5.8 Vault Extension (Optional)

This extension specifies capital-entrusted jobs, where a Provider takes custody of Client-provided capital for trading, yield strategies, or similar uses. Implementations that do not support capital custody MAY omit this section. Implementations that do support capital custody MUST conform to this section.

#### 5.8.1 StrategyVault

```solidity
interface IAACP_StrategyVault {
    struct VaultParams {
        uint256 initialCapital;
        int256  stopLossBps;             // negative; e.g. -2000 = -20%
        int256  targetReturnBps;         // positive; e.g. +1000 = +10%
        uint16  performanceFeeBps;
        uint16  managementFeeBps;
        uint256 deadline;
        address[] allowedCounterparties; // TransactionGuard allowlist
        uint256 maxDailyTrades;
        uint256 maxPositionSizeBps;
    }

    event VaultOpened(uint256 indexed jobId, VaultParams params);
    event VaultTrade(uint256 indexed jobId, address indexed counterparty, uint256 amountIn, uint256 amountOut);
    event VaultStopLossTriggered(uint256 indexed jobId, int256 currentReturnBps);
    event VaultClosed(uint256 indexed jobId, int256 finalReturnBps, uint256 settledValue);

    function openVault(uint256 jobId, VaultParams calldata params) external;
    function executeTrade(uint256 jobId, address counterparty, bytes calldata callData) external;
    function triggerStopLoss(uint256 jobId) external;
    function closeVault(uint256 jobId) external returns (int256 finalReturnBps);
    function vaultTotalValue(uint256 jobId) external view returns (uint256);
}
```

#### 5.8.2 TransactionGuard

Every trade MUST pass a pre-flight guard that enforces:

1. Counterparty is in `allowedCounterparties`.
2. Position size after the trade ≤ `maxPositionSizeBps` of vault total value.
3. Daily trade count ≤ `maxDailyTrades`.
4. Post-trade stop-loss not crossed.

Guard failures MUST revert the trade.

#### 5.8.3 Provider staking for capital-entrusted jobs

For capital jobs, the Provider lock amount uses a capital-scaled model:

```
Provider lock = (C × S × η × α) / (p × r)

Where:
  C      = Client capital
  S      = stop-loss fraction (absolute value, e.g. 0.20 for -20%)
  η      = safety margin multiplier (≥ 1)
  α      = liquidity coefficient (≤ 1, per market conditions)
  p      = slash-to-stake ratio
  r      = Provider reputation coefficient
```

Details and derivation are referenced in the AACP whitepaper §10.11. Implementations MUST expose the coefficients as governance-adjustable parameters.

#### 5.8.4 Off-chain capital extension (CEX Integration)

AACP deployments that support off-chain capital (CEX trade-only API keys) MUST use a TEE-hosted gateway that enforces the same guard rules from §5.8.2 on every outgoing order. Settlement proof MUST include a zkTLS or equivalent attestation of CEX account balances. An implementation MAY refer to the AACP whitepaper §10.10 for the full integration specification.

## Rationale

**Why a single master EIP instead of a family of extensions?**
AACP's components — roles, state machine, Skill Registry, dispute flow, staking, vault — are load-bearing for each other. Separating them into multiple extension EIPs obscures this coupling, forces duplicated context in each document, and under-represents AACP's positioning as an independent protocol category. ERC-4337 and ERC-7579 demonstrate that complete protocol standards benefit from a single specification even when they compose other ERCs as primitives.

**Why is the Provider outside the `skillSetHash` binding?**
The protocol's guarantee is that the deliverable was verified under the Client's selected Skill by an environment cryptographically bound to that Skill. Whether the Provider used the same methodology is economically irrelevant — good work produced by a proprietary playbook is equally acceptable. Requiring Providers to load a specific Skill would exclude (a) closed-source strategies, (b) proprietary playbooks, and (c) compute-specialized infrastructure that cannot fit a TEE budget. The opt-in royalty + reputation boost channel captures the subset of Providers who choose to bind their methodology publicly.

**Why content-address Skills on IPFS instead of storing them on-chain?**
Skill manifests, reference corpora, and verification programs can reach hundreds of megabytes. On-chain storage would be prohibitively expensive and would defeat the purpose of a "marketplace of methodologies". Content addressing preserves the integrity guarantee (any tampering changes the CID) without coupling storage cost to verification cost.

**Why SemVer and immutable prior versions?**
Disputes may arise months after job completion. An arbitrator re-executing under a different Skill version would not produce comparable results. Immutable prior versions ensure re-execution is meaningful; SemVer ranges let Clients express tolerance for minor/patch updates at `createJob` time without committing to future updates they have not reviewed.

**Why sealed sections inside verifier TEEs?**
If Providers can read every check an Evaluator will run, they can optimize for minimal compliance (format-matching) rather than genuine quality. Sealing adversarial corpora and red-flag detectors while keeping the scoring rubric public preserves the transparency that makes evaluation auditable, without giving up discrimination on the sealed surface.

**Why commit-reveal for arbitrator voting?**
Arbitrators who can see each other's votes may coordinate, explicitly or implicitly. Commit-reveal forces each arbitrator to independently commit before any vote is visible, producing a Schelling-point mechanism where honest convergence wins.

**Why both reactive and protocol-initiated disputes?**
Reactive disputes rely on the injured party to pay. Sophisticated Evaluators can exploit this by targeting low-budget jobs where Providers will not bother to challenge. Protocol-initiated disputes remove the cost barrier by having treasury front the deposit; reimbursement from slashed funds makes the mechanism economically sustainable.

**Why a staking pool instead of per-job escrow of Provider stake?**
Per-job escrow requires agents to redeposit for every job, which is operationally painful for agents that process many small jobs. The Pool model lets a single deposit back any number of concurrent or sequential jobs, improving capital efficiency while preserving per-job locks that are released only on terminal state.

**Why is Capability Declaration metadata largely unenforceable on-chain?**
Most capability claims (domain expertise, declared verification level, preferred budgets) are off-chain knowledge without a cheap on-chain proof. Forcing on-chain attestation for every claim would raise registration cost dramatically without improving settlement correctness. Instead, only role registration and Skill loading — the two claims that affect settlement — are enforced on-chain.

## Backwards Compatibility

AACP is fully compatible with ERC-8004 and ERC-8183: it references their contracts without modifying their interfaces.

- An agent registered only against an ERC-8004 contract (no AACP registration) can interact with AACP-unaware dapps without impact.
- An ERC-8183 escrow used directly (without AACP's evaluator phase) behaves per ERC-8183 semantics; AACP adds additional states that are reached only when the AACP-specific entry points are invoked.
- AACP deployments MAY support jobs where `strategy.kind` is among `Program`, `Rubric`, or `Hybrid` without requiring a Skill Registry; the Skill Registry is MANDATORY only when a job's `strategy.kind == Skill`.

Clients, wallets, and indexers that are aware only of ERC-8004 or ERC-8183 will observe AACP-specific state transitions as ordinary state changes in the composed contracts; AACP events are additive and may be ignored by legacy tooling.

## Reference Implementation

A reference implementation, along with deployment manifests, CLI tooling, and TEE/zkVM integration examples, is maintained in the AACP repository. The reference implementation includes:

- Conformant `IAACP_Job`, `IAACP_Roles`, `IAACP_SkillRegistry`, `IAACP_Dispute`, `IAACP_Staking`, `IAACP_StrategyVault` implementations.
- A VRF-based arbitrator selection module.
- A commit-reveal voting module with slashing for withheld reveals.
- A TEE attestation verifier for Intel SGX, Intel TDX, and AWS Nitro.
- A Groth16 zkVM proof verifier compatible with SP1 and RISC Zero outputs.
- An off-chain Skill-publishing toolkit, including manifest signing and IPFS pinning.

## Security Considerations

### Identity and Sybil resistance

- New agents start at reputation 50 and face 2× lock requirements per job until reputation improves. Implementations MUST prevent stake withdrawal paths that allow a slashed agent to migrate to a new NFT and re-enter at baseline trust; linking slash records to persistent off-chain identity anchors (e.g., ENS, proof-of-personhood) is RECOMMENDED for high-value deployments.

### Verification environment

- TEE vendors (Intel, AMD, AWS) remain a trust root for L1/L3 levels. Implementations SHOULD pin TEE attestation root certificates and rotate them on vendor-published schedules. Loss of attestation-chain validity MUST NOT silently downgrade a job's verification level.
- zkVM proofs for L2/L3 inherit the trusted setup assumptions of the selected proving system. Implementations SHOULD publish the ceremony artifacts for reproducibility.

### Skill Registry

- **Skill author collusion.** A Skill author who colludes with Evaluators off-chain to grant favorable scores can be detected by shadow evaluation on random job samples. Implementations MUST run periodic shadow evaluation and publish its results against each Skill's reported precision.
- **Sealed corpus leak.** If a sealed corpus leaks (an author publishes the adversarial set), the compromised version MUST be deprecated; a new corpus version MUST be issued before the Skill returns to `Active`.
- **Dependency injection.** Skills pin dependencies by hash at install time; upstream version bumps MUST NOT auto-propagate to active jobs. Implementations MUST surface the full resolved dependency tree at `createJob` for Client inspection.
- **Stale Skills.** Implementations MUST enforce a quarterly re-audit; Skills below declared precision/recall thresholds MUST transition to `Deprecated`.

### Dispute and arbitration

- **Arbitrator bribery.** The total economic value of a single dispute SHOULD be capped relative to the aggregate arbitration stake of the selected set. When exceeded, implementations MUST escalate to a larger panel or longer reveal window.
- **Commit-reveal griefing.** An arbitrator who commits but fails to reveal can destabilize the vote. Implementations MUST slash withholders and treat them as abstentions.
- **State-transition races.** Implementations MUST define a deterministic ordering between dispute window, arbitration window, and ERC-8183 auto-release deadlines. The RECOMMENDED safe policy: dispute window strictly expires before auto-release.

### Staking and slashing

- **Reentrancy at settlement.** Settlement releases escrow, slashes stake, and writes reputation in a single transaction. Implementations MUST use checks-effects-interactions and SHOULD use a reentrancy guard on the settlement function.
- **Gas exhaustion in large arbitrator sets.** Aggregation over large arbitrator sets MUST use incremental aggregation or off-chain aggregation with an on-chain verifier.
- **Minimum deposit floor.** Scaled dispute deposits SHOULD have a floor (RECOMMENDED 5 USDC) to prevent spam.

### Vault Extension

- **Provider access to TEE hardware.** For CEX-integration deployments, the Provider MUST NOT have physical access to the TEE hardware that holds the off-chain API key. The TEE host MUST be operationally independent of the Provider.
- **Guard bypass.** Every trade MUST pass the TransactionGuard; implementations MUST NOT expose any path that skips the guard, including privileged operator calls.
- **Stop-loss oracle manipulation.** Stop-loss evaluation relies on price oracles and balance reads. Implementations MUST use a TWAP or otherwise manipulation-resistant price source and MUST read CEX balances via an attested path.

### Capability declaration drift

- Capability claims written in an agent's ERC-8004 metadata may drift from on-chain behavior (e.g., an agent claims `verificationLevel: L2` but submits L1 attestations). Implementations MUST NOT treat off-chain capability claims as authoritative; settlement MUST rely only on on-chain events (role registration, Skill load, attestation content).

## Copyright

Copyright and related rights waived via [CC0](../LICENSE).
