---
eip: <to be assigned>
title: Capital-Entrusted Agent Vault
description: A standard interface for vaults that delegate scoped execution rights to autonomous agents while guaranteeing assets cannot leave the vault's control perimeter.
author: termix-it (@termix-it) <termixai@tuta.io>
discussions-to: <URL>
status: Draft
type: Standards Track
category: ERC
created: 2026-04-15
requires: 20, 165, 721
---

## Abstract

This EIP specifies a minimal on-chain interface for **Agent Vaults** — contracts that hold a principal's capital and grant an autonomous agent a scoped execution right to interact with a whitelisted set of external protocols, while cryptographically guaranteeing that assets cannot leave the vault's control perimeter. The standard unifies the `execute(target, data)` pattern, transaction guard semantics, stop-loss enforcement, and portfolio valuation, so that agent frameworks, risk dashboards, insurance providers, and UIs can interoperate across vault implementations.

## Motivation

A growing class of autonomous agents needs to **operate** a principal's capital rather than merely deliver an artifact: DeFi yield agents, on-chain market makers, portfolio rebalancers, procurement bots, liquidity managers. Today each project reinvents this wheel using Safe Modules, bespoke guard contracts, or custom proxies. Consequences:

- Shared tooling (risk dashboards, monitoring, insurance underwriters) cannot front-run implementations, so each project re-builds observability from scratch.
- Agent frameworks integrate separately with each vault variant.
- Principals cannot compare risk guarantees across competing agents, because "safety" is expressed in different primitives in every project.
- Safety-critical logic (stop-loss, whitelist enforcement, valuation) is re-audited per project, multiplying audit cost and attack surface.

A standard vault interface — in the spirit of ERC-4626 for yield vaults, but oriented around **delegated execution** rather than deposit/share accounting — enables an ecosystem of pluggable, auditable agent containers. Implementation economics (performance fees, multi-principal pooling, subscription models) remain free; the standard fixes only the **safety invariants** that must be uniform across any trustworthy vault.

## Specification

The key words "MUST", "MUST NOT", "REQUIRED", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

### Vault Interface

```solidity
interface IAgentVault /* is IERC165 */ {
    // ---------- Errors ----------
    error NotAgent();
    error DeadlinePassed();
    error GuardRejected(address target, bytes4 selector, bytes reason);
    error StopLossTriggered(uint256 valueAfter, uint256 stopLoss);
    error WindDownOnly();
    error DelegateCallForbidden();

    // ---------- Events ----------
    event Executed(
        address indexed agent,
        address indexed target,
        bytes4  indexed selector,
        bytes   data,
        uint256 valueAfter
    );
    event GuardRejectedEvent(address indexed target, bytes4 indexed selector, bytes reason);
    event StopLossHit(uint256 valueAfter, uint256 stopLoss);
    event WindDownStarted(address indexed initiator, uint256 timestamp);
    event Settled(uint256 finalValue, uint256 principalReturn, uint256 agentPerformanceFee);

    // ---------- Core: delegated execution ----------

    /// @notice Execute a call on an external protocol on behalf of the principal.
    /// @dev MUST revert if: msg.sender != agent(), block.timestamp >= deadline(),
    ///      guard().check() returns false, the call uses delegatecall semantics,
    ///      or post-call totalValue() < stopLoss().
    function execute(address target, bytes calldata data)
        external
        payable
        returns (bytes memory result);

    /// @notice Atomic batched execute. Any sub-call revert reverts the entire batch.
    function executeBatch(address[] calldata targets, bytes[] calldata datas)
        external
        payable
        returns (bytes[] memory results);

    // ---------- Introspection ----------

    function agent()          external view returns (address);
    function principal()      external view returns (address);
    function guard()          external view returns (address);
    function deadline()       external view returns (uint256);
    function stopLoss()       external view returns (uint256);
    function principalAsset() external view returns (address);

    /// @notice Total value of all assets controlled by the vault, denominated
    ///         in the principal asset via the vault's valuation policy.
    function totalValue() external view returns (uint256);

    /// @notice True if the vault has entered wind-down mode.
    function inWindDown() external view returns (bool);

    // ---------- Lifecycle ----------

    /// @notice Permissionless. Triggers wind-down if totalValue() < stopLoss()
    ///         or block.timestamp >= deadline(). MUST be idempotent.
    function poke() external;

    /// @notice Finalizes the job. Callable after deadline or after wind-down
    ///         has completed unwinding. Distributes principal, performance fee,
    ///         and any residuals per vault policy.
    function settle() external returns (uint256 finalValue);
}
```

### Guard Interface

```solidity
interface IAgentVaultGuard {
    struct Permission {
        address target;
        bytes4  selector;
    }

    /// @notice Called by the vault before forwarding `target.call(data)`.
    /// @return allowed True if the call is permitted.
    /// @return reason  Opaque rejection reason when allowed == false.
    function check(address target, bytes calldata data)
        external
        view
        returns (bool allowed, bytes memory reason);

    /// @notice Enumerate permitted (target, selector) pairs for off-chain
    ///         inspection by principals and auditors before job acceptance.
    function whitelist() external view returns (Permission[] memory);
}
```

### Mandatory Safety Invariants

A conforming vault MUST reject, **at the vault level and regardless of guard response**, any execution whose decoded calldata would:

1. Call `IERC20.transfer(to, …)` or `IERC20.approve(spender, …)` where `to` or `spender` is not present in `guard().whitelist()`.
2. Call `IERC721.transferFrom`, `IERC721.safeTransferFrom`, `IERC1155.safeTransferFrom`, or `IERC1155.safeBatchTransferFrom` where the `from` argument is the vault address.
3. Use `delegatecall` as the invocation semantic. `execute()` MUST use `call`.
4. Target any address not present in `guard().whitelist()`.

These invariants are the **last line of defense** and MUST NOT be relaxed by the guard. The guard may further restrict execution but may not weaken these rules.

### Stop-Loss Semantics

After every successful `execute()` or `executeBatch()`:

1. The vault MUST compute `totalValue()`.
2. If `totalValue() < stopLoss()`, the entire call MUST revert with `StopLossTriggered`.

Additionally, any caller MAY invoke `poke()`. If at the time of `poke()`:

- `totalValue() < stopLoss()`, or
- `block.timestamp >= deadline()`,

the vault MUST enter wind-down mode (`inWindDown() == true`) and MUST emit `WindDownStarted`. Once in wind-down, `execute()` is restricted to calls that (a) reduce non-principal asset balances and (b) result in non-decreasing principal-asset balances. The guard's whitelist and the mandatory invariants above enforce this together.

### Valuation Requirements

`totalValue()` MUST return a value denominated in `principalAsset()`. Implementations:

- SHOULD use Chainlink price feeds for blue-chip assets with a staleness bound ≤ 1 hour.
- SHOULD use time-weighted average prices (TWAP) with a window ≥ 30 minutes for long-tail assets.
- SHOULD require two independent price sources to agree within 5% for any asset whose mis-pricing could cross the stop-loss.
- MAY expose a non-normative `valuationPolicy()` view describing their oracle set.
- MUST revert `totalValue()` rather than return a stale or single-source value when oracle conditions are not met, if the resulting uncertainty could mask a stop-loss breach.

### ERC-165

A contract claiming support MUST return `true` from `supportsInterface()` for `type(IAgentVault).interfaceId`.

### Interaction with ERC-8004 and ERC-8183 (Non-Normative)

This EIP is independent of any particular identity or job-escrow standard. When used with ERC-8004 agent identity, the `agent()` address SHOULD be an address derivable from an ERC-8004 Agent NFT, so reputation and staking systems can attribute vault outcomes to the agent. When used with ERC-8183 job escrow, a management fee MAY flow through the standard ERC-8183 escrow while the vault's performance fee is paid at `settle()`. Neither mapping is required for conformance.

## Rationale

**Why a minimal interface rather than a full implementation?**
Agent vault economics (performance fees, multi-principal pooling, subscription models, treasury-backed insurance) vary too much to standardize. The interface isolates the *safety invariants* — scoped execution, stop-loss, whitelist, valuation — which must be uniform across trustworthy vaults, and leaves economic policy to implementations.

**Why `execute(target, data)` instead of per-protocol adapters?**
Per-protocol adapters require the vault to ship upgrades whenever a new DeFi protocol launches. The generic call + guard pattern lets principals opt into new protocols by extending the whitelist, with the vault unchanged. This is the same design trade-off made by Safe Modules and cowswap-style execution sandboxes.

**Why a separate Guard contract?**
Decoupling enables:

- Shared guards across vaults (e.g., a DAO-curated "audited protocols" guard that many vaults reference).
- Upgradable whitelists (where policy permits) without touching vault code.
- Guard-level innovations (ML-based anomaly detection, attestation-based rules) without forking the vault standard.

**Why hardcoded `delegatecall` rejection?**
`delegatecall` can rewrite vault storage, defeating every other invariant simultaneously. No legitimate DeFi interaction needs it; banning it at the interface level eliminates an entire vulnerability class in one rule.

**Why enumerate the whitelist on-chain?**
Principals inspecting a vault before accepting an agent need to see the full permission surface, not a hash of it. On-chain enumeration lets wallets and risk dashboards render a concrete "this agent can do X, Y, Z" disclosure.

**Why require valuation in the principal asset?**
Stop-loss, performance fees, and settlement all need a single unit of account. Forcing the vault to denominate valuation eliminates cross-unit ambiguity when composing vaults with off-chain tooling.

**Why `poke()` instead of a built-in keeper?**
Anyone — Chainlink Automation, Gelato, the principal, or a disinterested third party — can trigger wind-down. Permissionless `poke()` avoids coupling the standard to a specific automation vendor and supports bounty-based monitoring.

**Why atomic `executeBatch` as a first-class operation?**
Many DeFi strategies (flash-loan-assisted rebalances, multi-leg position opens) rely on atomicity. Without a batch primitive, agents would route through an external multicall that bypasses the vault's per-call invariants.

## Backwards Compatibility

No existing EIP defines delegated-execution vaults. This interface does not conflict with ERC-4626 (yield vaults) — the two can co-exist on the same contract when that makes sense. Existing Safe Modules and custom agent containers can wrap this interface as an adapter with no modification to the underlying wallet.

## Reference Implementation

See `contracts/StrategyVault.sol` and `contracts/TransactionGuard.sol` in the AACP reference repository. The reference implementation includes:

- Constant-time whitelist checks via bitmap lookups.
- Chainlink + TWAP dual-oracle valuation with per-asset staleness bounds.
- `poke()`-driven wind-down with a permissionless bounty paid from treasury.
- ERC-4626 share accounting on top of the `IAgentVault` safety layer (illustrative composition).

## Security Considerations

- **Guard correctness is load-bearing.** A misconfigured guard (e.g., whitelisting an unaudited protocol) permits losses within the stop-loss envelope. Vaults SHOULD expose `guard().whitelist()` in a form principals can inspect before accepting the job, and SHOULD make the whitelist immutable for the vault's lifetime unless a principal-approved upgrade path is used.
- **Hardcoded invariants are the final backstop.** The guard MAY be compromised or upgraded; the vault-level invariants MUST NOT depend on guard honesty. An auditor should be able to verify the four mandatory invariants without reading any guard source.
- **Oracle manipulation** is the primary attack surface against `totalValue()` and therefore against stop-loss. Implementations MUST use at least two independent price sources for any asset with non-trivial exposure, and SHOULD reject when sources diverge beyond a configured threshold.
- **Permissionless `poke()` griefing.** Since anyone can trigger wind-down, implementations MUST only enter wind-down when the stop-loss or deadline condition is *actually* met at the time of the call, not merely claimed by the caller.
- **Reentrancy.** `execute()` forwards arbitrary calldata; implementations MUST use checks-effects-interactions and a reentrancy guard on state-mutating paths. Batched calls require special care — reentrancy through one sub-call into another MUST be blocked.
- **Agent key compromise.** The `agent()` address is typically a hot wallet whose key may be stolen. The stop-loss + whitelist together bound the blast radius (attacker can at worst execute whitelisted operations until stop-loss triggers); principals SHOULD size capital accordingly and SHOULD prefer short deadlines for high-value vaults.
- **Oracle staleness at wind-down.** If oracles are stale precisely when wind-down begins, valuation may be wrong. Implementations SHOULD treat stale-oracle wind-down as a pause-and-dispute event rather than settling on suspect data.
- **Upgradability.** If the vault itself is upgradable, the principal's effective safety guarantees depend on the upgrade authority. This standard RECOMMENDS immutable per-job vaults, or time-locked upgrades with principal veto.
- **Re-entrancy via ERC-777 / ERC-1363 hooks.** Tokens with transfer hooks can re-enter `execute()` during settlement accounting. Implementations MUST treat transfer hooks as untrusted external calls.
- **Griefing via whitelist bloat.** An overlarge whitelist increases guard gas cost and attack surface. Principals SHOULD favor minimal whitelists; tooling SHOULD flag vaults with whitelists exceeding sane thresholds.
- **Insurance.** This standard does not prescribe an insurance mechanism; third-party insurance providers can price risk directly from the standardized invariants.

## Copyright

Copyright and related rights waived via [CC0](../LICENSE).
