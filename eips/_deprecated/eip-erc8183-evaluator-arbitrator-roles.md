---
eip: <to be assigned>
title: Evaluator and Arbitrator Roles for ERC-8183 Jobs
description: Extends the ERC-8183 job escrow state machine with Evaluator and Arbitrator roles and the state transitions required for programmatic verification and multi-party dispute resolution.
author: termix-it (@termix-it) <termixai@tuta.io>
discussions-to: <URL>
status: Draft
type: Standards Track
category: ERC
created: 2026-04-15
requires: 8183
---

## Abstract

ERC-8183 standardizes a two-party (Client / Provider) job escrow state machine. This EIP adds two first-class roles — **Evaluator** and **Arbitrator** — and the state transitions required for independent verification and multi-party dispute resolution. The extension is fully additive: any ERC-8183 implementation remains conformant, and jobs that do not use the new roles follow the original state machine unchanged.

## Motivation

ERC-8183 requires Clients to directly approve or reject Provider deliverables. In practice, this is insufficient for autonomous agent commerce:

- **Client competence.** A Client may not be qualified to evaluate agent output — e.g., a non-programmer Client receiving Solidity code, or a business user receiving a data pipeline they cannot inspect.
- **Client absence.** A Client may be offline, slow, or adversarial, trapping Provider compensation indefinitely.
- **No standard dispute path.** Today, disputes have no standardized resolution inside the job state machine. Resolution happens through off-protocol social or legal channels, which do not scale to agent-to-agent commerce.
- **No hook for programmatic verification.** zkVM-, TEE-, and rubric-based verification systems have nowhere to attach — they must be re-wired into every ERC-8183 deployment separately.

A standardized third-party verification and dispute layer enables:

- Specialized Evaluator markets, where a non-party verifies deliverables against Client-provided criteria for a fee.
- Consistent dispute mechanics across platforms, so tooling (indexers, wallets, insurance) can render dispute state uniformly.
- Drop-in integration points for programmatic verification: zkVM proofs, TEE attestations, semantic grading panels, or human rubric scoring.

## Specification

The key words "MUST", "MUST NOT", "REQUIRED", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

### Role Identifiers

```solidity
bytes32 constant ROLE_EVALUATOR  = keccak256("ERC8183.EVALUATOR");
bytes32 constant ROLE_ARBITRATOR = keccak256("ERC8183.ARBITRATOR");
```

### Extended State Machine

This EIP introduces three new job states and the transitions among them. All new states are reachable **only** from the existing ERC-8183 `Submitted` state. The original `Submitted → Completed` and `Submitted → Rejected` transitions remain valid for jobs that do not opt into this extension.

```
                          ┌──────────────┐
                          │  Submitted   │  (from ERC-8183)
                          └──────┬───────┘
                                 │ assignEvaluator
                                 ▼
                          ┌──────────────┐
                          │  InReview    │  ← new state
                          └──┬────────┬──┘
                 approve     │        │   reject
                             ▼        ▼
                       ┌──────────┐ ┌──────────┐
                       │Completed │ │ Disputed │  ← new state
                       └──────────┘ └────┬─────┘
                                         │ openArbitration
                                         ▼
                                  ┌──────────────┐
                                  │ InArbitration│  ← new state
                                  └──┬────────┬──┘
                            upheld   │        │  overturned
                                     ▼        ▼
                               ┌──────────┐ ┌──────────┐
                               │Completed │ │ Refunded │
                               └──────────┘ └──────────┘
```

### Interface

```solidity
interface IERC8183_Verification /* extends IERC8183 */ {
    // ---------- Events ----------
    event EvaluatorAssigned(uint256 indexed jobId, address indexed evaluator);
    event JobReviewStarted(uint256 indexed jobId, address indexed evaluator);
    event JobApproved(uint256 indexed jobId, address indexed evaluator, bytes32 verdict);
    event JobRejected(uint256 indexed jobId, address indexed evaluator, bytes32 verdict, string reason);
    event DisputeOpened(uint256 indexed jobId, address indexed challenger, uint256 deposit);
    event ArbitrationOpened(uint256 indexed jobId, bytes32 indexed arbitratorSetId);
    event ArbitrationVoteSubmitted(uint256 indexed jobId, address indexed arbitrator, bool upheld, bytes32 evidence);
    event ArbitrationResolved(uint256 indexed jobId, bool upheld, address[] arbitrators);

    // ---------- Evaluator phase ----------

    /// @notice Client assigns an Evaluator to a Submitted job.
    /// @dev MUST be called by the Client. MUST transition the job to InReview
    ///      and emit JobReviewStarted. The Client MAY self-assign to preserve
    ///      the original two-party behavior.
    function assignEvaluator(uint256 jobId, address evaluator) external;

    /// @notice Evaluator approves the submission. MUST transition to Completed
    ///         and trigger the standard ERC-8183 payment release.
    /// @param verdict Optional commitment hash to off-chain evaluation evidence.
    function approveJob(uint256 jobId, bytes32 verdict) external;

    /// @notice Evaluator rejects the submission. MUST transition to Rejected
    ///         (per ERC-8183) unless a dispute is opened within the dispute window.
    function rejectJob(uint256 jobId, bytes32 verdict, string calldata reason) external;

    // ---------- Dispute phase ----------

    /// @notice Provider or Client opens a dispute against an Evaluator's
    ///         approval or rejection. Requires a deposit, forfeited if the
    ///         dispute is ruled against the challenger.
    /// @dev MUST transition to Disputed and freeze escrow distribution.
    function openDispute(uint256 jobId) external payable;

    /// @notice Moves a Disputed job into arbitration under a selected
    ///         arbitrator set. Selection policy is implementation-defined.
    /// @dev MUST transition to InArbitration.
    function openArbitration(uint256 jobId, bytes32 arbitratorSetId) external;

    /// @notice Arbitrator submits a verdict on whether the Evaluator's
    ///         decision should be upheld.
    /// @dev Implementations MUST aggregate verdicts (majority, stake-weighted,
    ///      schelling, etc.) and resolve once the aggregation threshold is met.
    function submitArbitrationVote(uint256 jobId, bool upholdEvaluator, bytes32 evidence) external;

    // ---------- Views ----------

    function evaluatorOf(uint256 jobId)         external view returns (address);
    function arbitratorsOf(uint256 jobId)       external view returns (address[] memory);
    function disputeDepositOf(uint256 jobId)    external view returns (uint256);
    function verdictOf(uint256 jobId)           external view returns (bytes32 verdict, bool finalized);
}
```

### Role Registry (Optional but RECOMMENDED)

Implementations SHOULD expose a registry so Clients can discover eligible Evaluators and Arbitrators:

```solidity
interface IERC8183_RoleRegistry {
    function registerAsEvaluator(bytes32[] calldata capabilities) external;
    function registerAsArbitrator() external;
    function isEvaluator(address account, bytes32 capability) external view returns (bool);
    function isArbitrator(address account) external view returns (bool);
}
```

Registries MAY impose stake, reputation, or KYC requirements. This EIP does not prescribe any particular gating mechanism.

### State Transition Rules

1. **`Submitted → InReview`** via `assignEvaluator`. Callable only by the Client, before any terminal transition. `evaluatorOf(jobId)` MUST return the assigned address after the call.
2. **`InReview → Completed`** via `approveJob`. Callable only by `evaluatorOf(jobId)`. MUST trigger the ERC-8183 payment release. The `verdict` hash is recorded on-chain; the pre-image MAY be published off-chain.
3. **`InReview → Rejected`** via `rejectJob`. Callable only by `evaluatorOf(jobId)`. After the dispute window elapses without an `openDispute` call, the job settles per ERC-8183's rejection semantics.
4. **`{Completed,Rejected} → Disputed`** via `openDispute`, only within a dispute window defined by the implementation (RECOMMENDED: 48 hours). Requires `msg.value == disputeDepositOf(jobId)`. MUST freeze further escrow movement until arbitration resolves.
5. **`Disputed → InArbitration`** via `openArbitration`. May be permissionless or restricted depending on implementation; an implementation MAY also transition automatically upon `openDispute`.
6. **`InArbitration → Completed`** when aggregated arbitrator verdicts uphold the Evaluator's original approval, or when a previously rejected job is overturned into a completion.
7. **`InArbitration → Refunded`** when aggregated verdicts overturn a previously approved job, causing funds to return to the Client.

Implementations MUST define a deterministic ordering policy when dispute windows and deadlines overlap (e.g., *the dispute window always expires strictly before escrow auto-release*).

## Rationale

**Why add new states instead of overloading `Submitted`?**
Downstream tooling (indexers, dashboards, wallets) benefits from an unambiguous signal of which phase a job is in. Overloading `Submitted` would force every consumer to inspect auxiliary storage or event logs to reconstruct state, defeating the purpose of a state machine.

**Why optional, not mandatory, Evaluators?**
Simple jobs where Client and Provider can resolve directly should not pay the overhead of assigning a third party. Optional Evaluators preserve ERC-8183's two-party happy path while unlocking the three-party flow when needed.

**Why a `bytes32 verdict` hash?**
Evaluation evidence is often verbose — rubric scores, test outputs, model reasoning, zkVM proofs. Committing a hash on-chain while storing the body off-chain (or in an attached verifier proof) is the standard trade-off between integrity and gas cost. Consumers that need the full evidence fetch it from IPFS or a verifier.

**Why not prescribe arbitrator selection?**
Selection policies vary widely: round-robin from a staked pool, schelling-point voting, Kleros-style courts, reputation-weighted draws, VRF-based random selection. Prescribing one would foreclose legitimate design space. This EIP standardizes only the *shape* of arbitration — deposit, verdict aggregation, resolution — without dictating the mechanism.

**Why `arbitratorSetId` as `bytes32`?**
Some implementations select arbitrators lazily (on-chain randomness, VRF) and need a handle before the set is materialized. An opaque `bytes32` covers both eager selection (hash of the sorted address list) and lazy selection (VRF seed commitment).

**Why a dispute deposit?**
Free disputes permit griefing. A mandatory deposit proportional to job value bounds spam while remaining refundable for successful disputes. The deposit also provides the reward pool for honest arbitrators.

**Why not standardize slashing of Evaluators?**
Slashing economics are policy decisions: the slash percentage, the distribution of slashed funds, the relationship to stake-weighted reputation. Different platforms will make different trade-offs. This EIP exposes the **events** and **state** needed for slashing policies to plug in without codifying any specific curve.

**Why permit the Client to self-assign as Evaluator?**
This preserves exact backward compatibility with ERC-8183's two-party happy path: a job whose Evaluator is the Client behaves identically to a plain ERC-8183 job. It also avoids forcing a separate "legacy mode" flag.

## Backwards Compatibility

Fully additive. A contract conforming to both ERC-8183 and this extension SHOULD route `approveJob` through the same internal logic as the direct Client approval path, so downstream economic rules (escrow release, fee distribution, reputation updates) remain unchanged. ERC-8183 indexers that do not recognize the new events will still see the underlying escrow state transitions unchanged.

## Reference Implementation

Omitted from this EIP. See the AACP reference implementation for a fully integrated example including:

- VRF-based arbitrator selection from a staked pool with reputation gating.
- Commit-reveal arbitrator voting with slashing for votes against the majority.
- Scaled dispute deposits based on verification strategy and Evaluator track record.
- Protocol-initiated disputes triggered by monitoring oracles (where the protocol treasury covers the deposit).

## Security Considerations

- **Evaluator collusion.** A corrupt Evaluator can rubber-stamp garbage submissions or reject honest work. Implementations SHOULD require Evaluator stake and SHOULD feed dispute outcomes into Evaluator reputation. Without a downstream penalty loop, the Evaluator role degrades into a rubber stamp.
- **Dispute spam.** Free disputes enable griefing. The mandatory deposit in `openDispute` bounds spam; implementations SHOULD size it proportional to job value (RECOMMENDED: 2–5% of escrowed budget, adjusted by verification strategy difficulty).
- **Arbitrator bribery.** Larger arbitrator sets and random selection reduce collusion surface. Implementations SHOULD cap the economic value of any single arbitration relative to the aggregate stake of its arbitrator set; when the cap is exceeded, implementations SHOULD escalate (larger panel, longer reveal window, external appeal).
- **Reentrancy at resolution.** Resolution releases escrow, slashes stake, and updates reputation in a single transaction; implementations MUST use checks-effects-interactions and SHOULD use a reentrancy guard on the resolution function.
- **State-transition races.** Evaluators and disputants may interact near deadlines. Implementations MUST define a deterministic ordering between the dispute window, the arbitration window, and any ERC-8183 auto-release deadline. A common safe policy: dispute window strictly expires before auto-release.
- **Commit-reveal manipulation.** If arbitrators use commit-reveal, an arbitrator who withholds the reveal can destabilize aggregation. Implementations SHOULD slash the withholder and treat the vote as abstention.
- **Gas exhaustion in verdict aggregation.** Large arbitrator sets may exceed block gas limits in a naive aggregation. Implementations SHOULD use incremental aggregation (stream-style reduce) or an off-chain aggregation with on-chain verification.
- **Client self-assignment abuse.** A Client who self-assigns as Evaluator gains direct approval/rejection power. This matches plain ERC-8183 semantics but MAY be exploited to avoid independent review; implementations MAY restrict self-assignment for high-value jobs.
- **Oracle dependence for programmatic verification.** When the evaluator runs a zkVM or TEE program, implementations MUST ensure the verification program itself cannot be manipulated post-assignment (e.g., by pinning it at `assignEvaluator` time).
- **Dispute window and fund lock-up.** Extended dispute windows lock Provider funds. Implementations SHOULD balance dispute window length against Provider capital efficiency.

## Copyright

Copyright and related rights waived via [CC0](../LICENSE).
