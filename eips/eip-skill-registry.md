---
eip: <to be assigned>
title: Agent Skill Registry
description: A standalone registry standard for packaged, versioned, signed domain expertise ("Skills") that any AI-agent platform, protocol, or application can publish, resolve, and consume.
author: termix-it (@termix-it) <termixai@tuta.io>
discussions-to: <URL>
status: Draft
type: Standards Track
category: ERC
created: 2026-04-24
---

## Abstract

This EIP specifies the **Agent Skill Registry** — a domain-agnostic on-chain registry for distributing, resolving, and accounting the use of packaged domain expertise ("Skills"). A Skill is a signed, versioned, content-addressed manifest describing a methodology, rubric, verification programs, tool allowlists, and reference data. The registry provides:

1. A canonical Skill manifest schema compatible with the open [agentskills.io](https://agentskills.io) standard.
2. SemVer-anchored publication with immutable prior versions.
3. Discovery and resolution primitives for consumers (agents, frameworks, protocols).
4. An event-based **load declaration** mechanism that lets arbitrary consumers record which Skill they used for which task.
5. An **optional** royalty-distribution hook that lets payment systems forward per-use fees to Skill authors.
6. An emergent reputation surface (precision, recall, usage counts, overturn rate) aggregated from consumer-emitted events.

This registry is intentionally protocol-agnostic. It does not define roles, jobs, escrow, disputes, or any application workflow. It can be used by any on-chain protocol (including AACP) as a shared dependency, or by off-chain agent frameworks that wish to anchor their Skill catalog on-chain.

## Motivation

Specialized domain expertise — audit checklists, clinical review criteria, legal clause libraries, risk models, translation style guides — is today fragmented across firms, playbooks, and proprietary tools. No composable standard exists for packaging this expertise and pricing its re-use. As autonomous-agent systems adopt the Claude-Skill-style methodology-as-content model, the missing layer is an on-chain settlement and discovery registry that:

- Gives Skill authors a **persistent, signed, versioned identity** for each published methodology.
- Gives consumers (agents, verifiers, tools) **content-integrity guarantees** (any tampering changes the CID).
- Records **load events** so royalties and reputation can accumulate, whether the consumer is a smart contract (paying royalties on-chain) or an off-chain agent (recording usage for attribution).
- Stays **protocol-neutral** so that the same Skill can be consumed by different protocols — agent marketplaces, verification networks, evaluation platforms — without fragmentation.

A standalone registry is therefore analogous to ERC-165 or ERC-6551: a small, composable primitive that many higher-level protocols depend on, rather than a feature buried inside any one protocol.

## Specification

The key words "MUST", "MUST NOT", "REQUIRED", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

### 1. Skill Identity

Each Skill is identified by a pair `(id, version)` where:

- `id` = `keccak256(bytes(name))`, where `name` is the Skill's lowercase, hyphenated canonical name (`solidity-audit`, `clinical-trial-review`, etc.).
- `version` is a SemVer 2.0.0 version string, immutable once published.

A new version of the same Skill reuses the same `id`. Only the registered author of the first version (the "owner") MAY publish later versions.

### 2. Skill Manifest

The manifest is a YAML document stored off-chain (IPFS or Arweave); the registry commits only the `manifestCID` and selected structured fields.

```yaml
---
# identity
name: string                     # REQUIRED; lowercase, hyphenated
version: semver                  # REQUIRED; immutable per publication
description: string              # REQUIRED; one-paragraph purpose
author: address                  # REQUIRED; signs the manifest
signature: bytes                 # REQUIRED; secp256k1 signature over canonical manifest hash

# content (IPFS CIDs, all OPTIONAL except where noted)
skillCID: bytes32                # REQUIRED; SKILL.md body CID
referenceCorpusCID: bytes32      # OPTIONAL; public labelled samples
sealedCorpusCID: bytes32         # OPTIONAL; encrypted adversarial data
verificationPrograms:            # OPTIONAL
  - cid: bytes32
    hash: bytes32
    kind: "public-check" | "sealed-check"

# consumer affordances
allowedTools: [string, ...]      # OPTIONAL; tool allowlist for consumers
jobTypes: [string, ...]          # OPTIONAL; consumer-defined categorization
domain: [string, ...]            # REQUIRED; ≥ 1 domain tag

# dependencies (resolved at load time)
dependencies:
  - skillId@semver-range         # OPTIONAL; may be empty

# economics (OPTIONAL; if omitted, royalty hook MUST be disabled for this Skill)
economics:
  royaltyBps:        uint16      # basis points taken per load on consumer side
  royaltyTokens:     [address]   # OPTIONAL; if set, royalty is only collected in these tokens
  authorStake:       uint256     # in stakeToken (configured by registry deployment)
  minimumFee:        uint256     # OPTIONAL; per-load floor in stakeToken
  slashOnMaliceBps:  uint16      # typically 10000
  slashOnStaleBps:   uint16      # typically 2000

# audit thresholds (OPTIONAL; required for Active state if registry enforces shadow-test)
audit:
  shadowTestCorpusCID: bytes32
  minPrecisionPpm:     uint32
  minRecallPpm:        uint32
---
```

Consumers MAY extend the manifest with protocol-specific fields under a namespaced key (e.g., `x-aacp:`). The registry MUST preserve such extensions verbatim in the on-chain `manifestCID` commitment but MUST NOT interpret them.

### 3. Lifecycle

```
                 registerSkill                        activateSkill
                 ┌─────────────┐  shadow-test pass   ┌─────────────┐
  (new version)  │  ShadowTest │ ──────────────────▶ │   Active    │
  ─────────────▶ └──────┬──────┘                     └──────┬──────┘
                        │ shadow-test fail                  │ deprecateSkill
                        ▼                                   ▼
                 ┌─────────────┐                      ┌─────────────┐
                 │   Retired   │                      │ Deprecated  │
                 │ (never live)│                      └──────┬──────┘
                 └─────────────┘                             │ retireSkill
                                                             │ (after unlock window)
                                                             ▼
                                                      ┌─────────────┐
                                                      │   Retired   │
                                                      └─────────────┘
```

Registry deployments MAY elect to skip `ShadowTest` (immediate `Active` on `registerSkill`). Deployments that enforce shadow-testing MUST verify `reportedPrecisionPpm >= audit.minPrecisionPpm` and `reportedRecallPpm >= audit.minRecallPpm` before allowing the transition.

### 4. Registry Interface

```solidity
interface IAgentSkillRegistry {
    enum SkillState { Unknown, ShadowTest, Active, Deprecated, Retired }

    struct SkillDescriptor {
        bytes32 id;
        string  version;
        address author;
        bytes32 manifestCID;
        bytes32 skillCID;
        bytes32 referenceCorpusCID;
        bytes32 sealedCorpusCID;
        uint16  royaltyBps;
        uint256 authorStake;
        SkillState state;
    }

    // ========== Publication ==========

    event SkillRegistered(
        bytes32 indexed id,
        string  version,
        address indexed author,
        bytes32 manifestCID
    );
    event SkillShadowPassed(bytes32 indexed id, string version, uint32 precisionPpm, uint32 recallPpm);
    event SkillActivated  (bytes32 indexed id, string version);
    event SkillDeprecated (bytes32 indexed id, string version, string reason);
    event SkillRetired    (bytes32 indexed id, string version);

    /// @notice Register a new Skill or publish a new version.
    /// @dev Reverts if (a) manifest.name differs from the existing Skill's name,
    ///      (b) msg.sender is not the registered author of the first version,
    ///      (c) manifest.version is not strictly greater than all prior versions,
    ///      (d) manifest.signature does not verify against the manifest hash.
    ///      Locks `manifest.economics.authorStake` from the caller.
    function registerSkill(bytes calldata manifest, bytes calldata signature)
        external
        returns (bytes32 id, string memory version);

    /// @notice Transition a Skill version from ShadowTest to Active.
    /// @dev Callable only by the registry's shadow-test oracle (governance-configured).
    function activateSkill(
        bytes32 id,
        string calldata version,
        uint32 reportedPrecisionPpm,
        uint32 reportedRecallPpm
    ) external;

    function deprecateSkill(bytes32 id, string calldata version, string calldata reason) external;
    function retireSkill   (bytes32 id, string calldata version) external;

    // ========== Discovery ==========

    function skillOf(bytes32 id, string calldata version) external view returns (SkillDescriptor memory);
    function resolveSkillRange(bytes32 id, string calldata range)
        external view
        returns (string memory version, bytes32 manifestCID);
    function versionsOf(bytes32 id)
        external view
        returns (string[] memory versions, SkillState[] memory states);
    function skillsByDomain(string calldata domain, uint256 offset, uint256 limit)
        external view
        returns (bytes32[] memory ids);

    // ========== Load declaration ==========

    event SkillLoaded(
        bytes32 indexed id,
        string          version,
        address indexed consumer,
        bytes32         context,        // consumer-defined (e.g. jobId, sessionId, taskId)
        bytes32 indexed purposeTag,     // consumer-defined category (e.g. verify/execute/review)
        bytes           attestation     // OPTIONAL; consumer-defined authenticity proof
    );

    /// @notice Record that `msg.sender` loaded the given Skill for the given context.
    /// @dev The registry MUST emit SkillLoaded and MAY trigger the royalty hook (§5).
    ///      `context` and `purposeTag` are opaque to the registry; consumers SHOULD document
    ///      their conventions in their own specs.
    function declareLoad(
        bytes32 id,
        string calldata version,
        bytes32 context,
        bytes32 purposeTag,
        bytes calldata attestation
    ) external;

    // ========== Reputation ==========

    struct SkillReputation {
        uint32 precisionPpm;
        uint32 recallPpm;
        uint32 overturnRatePpm;
        uint64 usageCount;
    }

    function reputationOf(bytes32 id) external view returns (SkillReputation memory);

    /// @notice Record an emergent outcome for a Skill use.
    /// @dev Callable only by accounts that the registry's governance has whitelisted
    ///      as reputation sources (e.g., dispute contracts, shadow-test oracle).
    ///      The registry MUST aggregate these reports into SkillReputation.
    function reportOutcome(
        bytes32 id,
        string calldata version,
        bool passed,
        bool overturned
    ) external;
}
```

### 5. Royalty Hook (OPTIONAL)

Registry deployments MAY implement an on-chain royalty hook that consumers invoke at payment time. The hook is optional because not all consumers are smart contracts; an off-chain agent framework can emit `SkillLoaded` events for attribution without triggering any payment.

```solidity
interface IAgentSkillRoyaltyHook {
    /// @notice Distribute `amount` of `token` as royalty to the author of Skill (id, version).
    /// @dev The caller (typically a payment-routing contract in the consumer protocol)
    ///      MUST have previously approved `amount` of `token` to this registry.
    ///      The registry MUST forward exactly `amount` to the author recorded at
    ///      SkillRegistered time; intermediaries MAY charge a fee only if the
    ///      registry deployment's governance has enabled such a fee.
    function distributeRoyalty(
        bytes32 id,
        string calldata version,
        address token,
        uint256 amount
    ) external;

    event RoyaltyDistributed(
        bytes32 indexed id,
        string          version,
        address indexed author,
        address indexed token,
        uint256         amount,
        address         payer
    );
}
```

Consumers that do not use the hook MAY still emit `SkillLoaded` events for attribution. Consumers that use the hook but whose `token` is not in `manifest.economics.royaltyTokens` MUST receive a revert.

### 6. Reputation Aggregation

Reputation fields update as follows:

- `usageCount` — increments by 1 per `SkillLoaded` emission.
- `overturnRatePpm` — reportOutcome callers (authorized reputation sources) MUST report `overturned=true` when a prior use of this Skill was later invalidated (e.g., a dispute overturned an evaluation that loaded this Skill). The registry aggregates `(overturned / total)` over a configurable window.
- `precisionPpm` / `recallPpm` — authorized shadow-test oracles MAY post periodic measurements; the registry exposes the latest.

Reputation is a **signal**, not an enforcement mechanism. Consumer protocols decide how to use it (for Skill ranking, stake scaling, etc.).

### 7. Author Stake and Slashing

Author stake is locked in a `stakeToken` (registry-deployment-configured; typically a stablecoin). Conditions under which stake MAY be slashed:

- **Malicious Skill.** A registry-authorized audit committee determines that the Skill was constructed to benefit a specific party (collusion). Slash amount: `slashOnMaliceBps`.
- **Stale Skill.** A Skill's emergent precision falls below `audit.minPrecisionPpm` for two consecutive audit cycles and the author fails to publish a corrective version within the deprecation unlock window. Slash amount: `slashOnStaleBps`.

Slashed funds route to the registry treasury (which governance MAY further distribute to affected consumers). Implementations MUST emit `AuthorSlashed(id, version, amount, reason)` whenever slashing occurs.

## Rationale

**Why a standalone registry rather than a feature of a specific protocol?**
Domain verification expertise composes across consumers. The same `solidity-audit` Skill benefits a job marketplace (AACP), a code-review agent framework, a bug-bounty platform, and an insurance underwriter. Embedding the registry inside any single protocol forces all others to reinvent it or depend on a foreign protocol's version semantics. ERC-165 and ERC-6551 set the precedent that small, protocol-neutral primitives are best as independent EIPs.

**Why content-address manifests instead of storing them on-chain?**
Manifests include prompts, examples, reference corpora, and potentially verification programs — hundreds of kilobytes to hundreds of megabytes. On-chain storage is prohibitive. Content addressing preserves tamper-evidence (any change alters the CID) and lets off-chain infrastructure (IPFS, Arweave, S3 with integrity checks) carry the payload.

**Why SemVer and immutable versions?**
Disputes, audits, and reproducibility requirements may surface weeks or months after a Skill was loaded. Under mutable versioning, re-execution under "version 2.1.0" would yield different results depending on when the query is made. SemVer combined with per-version immutability gives consumers a stable anchor for "exactly what I loaded then."

**Why is the load declaration event generic rather than protocol-specific?**
A single registry should support consumers as diverse as smart contracts, off-chain agents, and third-party tooling. A generic `(context, purposeTag, attestation)` triple lets each consumer category document its own conventions (e.g., AACP uses `context = jobId`, `purposeTag = keccak256("verify")`) without the registry having to know.

**Why is the royalty hook optional?**
Not every consumer is a smart contract. An off-chain agent framework (Claude Code, Cursor, a custom runner) that loads a Skill for guidance has no on-chain payment surface. Forcing an on-chain royalty would either block off-chain adoption or force every off-chain loader to pay a gas fee per use. Making the hook optional lets on-chain consumers pay royalties natively while preserving off-chain attribution via events.

**Why no concept of "consumer role"?**
Roles belong to consumer protocols, not to a neutral registry. AACP defines Evaluator, Arbitrator, Provider. A medical-review protocol might define Reviewer, Moderator, Adjudicator. An off-chain framework has "the agent." The registry only needs to know *that* a consumer loaded a Skill, not *what kind* of consumer.

## Backwards Compatibility

This EIP introduces a new standalone contract family. It does not modify any existing standard.

- **With ERC-721 / ERC-1155.** Skills are not tokens; the registry emits its own events. Authors MAY additionally mint an ERC-721 to represent Skill authorship if desired.
- **With ERC-2981 (NFT Royalty Standard).** The Skill royalty hook is independent of ERC-2981; Skills are not NFTs and royalty semantics are per-load rather than per-transfer.
- **With ERC-8004 / ERC-8183 / AACP.** AACP references this EIP as a dependency; other consumer protocols may do the same.

## Reference Implementation

A reference implementation in Solidity, plus a JSON Schema for the manifest, a CLI publishing tool, and an off-chain indexer are maintained in the AACP reference repository under `eips/reference/skill-registry/`.

## Security Considerations

- **Authorship squatting.** Once a `name` is registered, only the original author can publish further versions. Name-squatting attacks (registering popular names first) are mitigated by `authorStake`: squatters lose stake if they publish low-quality Skills that never reach Active state. Registry deployments MAY impose namespace curation (e.g., domain-prefixed names).
- **Version-ordering attacks.** An author publishing version `2.0.0` immediately after `1.0.0` to starve consumers of the lower version MUST be prevented by the registry refusing to retire a version until its unlock window elapses.
- **CID drift.** IPFS CIDs may become unpinnable if no node hosts the content. Registry deployments SHOULD encourage authors to dual-pin to Arweave and SHOULD expose a `reportUnpinnedSkill` function so consumers can signal missing content.
- **Royalty token manipulation.** If a Skill accepts multiple royalty tokens, consumers could pay royalty in a manipulated-price token. The registry MUST pass `token` and `amount` as the author receives them; authors SHOULD restrict `royaltyTokens` to trusted stablecoins.
- **Sealed corpus leak.** If a sealed corpus leaks (the adversarial test set becomes public), the compromised version MUST be deprecated and a new corpus version issued before the Skill returns to Active.
- **Author key compromise.** If an author's signing key is compromised, the author MUST be able to recover authorship through a social-recovery or guardian process. Registry deployments SHOULD support this via EIP-1271 smart-account authors or a registry-level recovery function gated by a governance action.
- **Reputation whitewashing.** An author who publishes a low-quality Skill, lets it accumulate a bad reputation, then republishes under a new name, defeats Skill-level reputation. Registry deployments SHOULD track author-level reputation in addition to Skill-level.
- **Oracle dependency.** Shadow-test oracles and reputation sources are privileged. Their compromise is equivalent to the registry being compromised. Deployments SHOULD use multiple independent oracles with a k-of-n threshold.

## Copyright

Copyright and related rights waived via [CC0](../LICENSE).
