---
eip: <to be assigned>
title: Agent Capability Metadata Schema
description: A standardized JSON metadata schema for declaring autonomous agent capabilities, pricing, verification profile, and service terms on top of ERC-8004 Agent NFTs.
author: termix-it (@termix-it) <termixai@tuta.io>
discussions-to: <URL>
status: Draft
type: Standards Track
category: ERC
created: 2026-04-15
requires: 721, 8004
---

## Abstract

This EIP defines a canonical JSON schema — *Agent Capability Metadata* — for describing the capabilities, pricing, verification profile, and service-level terms of autonomous agents. The schema is returned from the `tokenURI()` of an ERC-8004 Agent NFT. A shared schema enables marketplaces, job routers, matching contracts, and cross-platform aggregators to discover and compose agents programmatically without writing a bespoke adapter for every agent framework.

## Motivation

ERC-8004 standardizes on-chain agent identity but leaves the metadata payload at `tokenURI()` unspecified. Today, every agent framework (ElizaOS, Virtuals, AgentKit, Morpheus, AACP, etc.) ships its own shape for that payload, with the following consequences:

- Clients cannot express queries like *"all agents that write Solidity, priced under 500 USDC, running on Base"* across platforms.
- Aggregators must maintain N per-platform adapters for N agent sources.
- Agents cannot port their identity metadata between ecosystems without manual re-authoring.
- Job-matching contracts cannot be written in a platform-agnostic way — they must hard-code platform-specific fields.

This mirrors the pre-ERC-721-metadata era of NFTs, when each marketplace implemented its own parser. A canonical metadata schema unblocks a composable agent discovery layer, analogous to what ERC-721 metadata did for NFT marketplaces.

## Specification

The key words "MUST", "MUST NOT", "REQUIRED", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

A contract claiming compliance with this EIP:

1. MUST implement ERC-8004.
2. MUST return, from `tokenURI(uint256)`, a URI that resolves to a UTF-8 JSON document.
3. That JSON document MUST validate against the JSON Schema defined in *§Schema Definition*.
4. MUST emit `AgentCapabilityUpdated` whenever the metadata content (not merely the URI) changes.

### Event

```solidity
event AgentCapabilityUpdated(
    uint256 indexed tokenId,
    string  newURI,
    uint256 schemaVersion
);
```

`schemaVersion` corresponds to the top-level `version` field of the JSON payload.

### Schema Definition

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Agent Capability Metadata",
  "type": "object",
  "required": ["name", "version", "capabilities", "pricing"],
  "properties": {
    "name":        { "type": "string", "maxLength": 128 },
    "version":     { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" },
    "description": { "type": "string", "maxLength": 2048 },
    "capabilities": {
      "type": "array",
      "minItems": 1,
      "items": { "$ref": "#/definitions/capability" }
    },
    "pricing":     { "$ref": "#/definitions/pricing" },
    "verification": { "$ref": "#/definitions/verification" },
    "jobTypes": {
      "type": "array",
      "items": { "enum": ["one-shot", "streaming", "capital-entrusted", "subscription"] }
    },
    "sla":      { "$ref": "#/definitions/sla" },
    "runtime":  { "$ref": "#/definitions/runtime" },
    "operator": { "type": "string", "description": "Optional human-readable operator identity or DID" },
    "endpoints": {
      "type": "object",
      "description": "Off-chain endpoints for bidding, negotiation, delivery.",
      "properties": {
        "bid":    { "type": "string", "format": "uri" },
        "submit": { "type": "string", "format": "uri" },
        "status": { "type": "string", "format": "uri" }
      }
    }
  },
  "definitions": {
    "capability": {
      "type": "object",
      "required": ["id"],
      "properties": {
        "id":    { "type": "string", "description": "Canonical capability tag, e.g. 'code.solidity'" },
        "level": { "enum": ["basic", "proficient", "expert"] },
        "tags":  { "type": "array", "items": { "type": "string" } }
      }
    },
    "pricing": {
      "type": "object",
      "required": ["currency", "model"],
      "properties": {
        "currency":          { "type": "string", "description": "ERC-20 address or ticker (USDC, ETH, USD)" },
        "model":             { "enum": ["fixed", "hourly", "performance", "subscription"] },
        "min":               { "type": "string", "description": "Decimal string, inclusive minimum" },
        "max":               { "type": "string", "description": "Decimal string, inclusive maximum" },
        "performanceFeeBps": { "type": "integer", "minimum": 0, "maximum": 10000 }
      }
    },
    "verification": {
      "type": "object",
      "properties": {
        "levels": {
          "type": "array",
          "minItems": 1,
          "items": { "enum": ["none", "tee", "zkvm", "tee+zkvm", "semantic", "human"] }
        },
        "teeAttestation": { "type": "string", "description": "Hex-encoded TEE attestation, if applicable" },
        "zkvm":           { "enum": ["sp1", "risc-zero", "jolt", "other"] }
      }
    },
    "sla": {
      "type": "object",
      "properties": {
        "maxConcurrentJobs": { "type": "integer", "minimum": 1 },
        "responseTimeSec":   { "type": "integer", "minimum": 0 },
        "availabilityBps":   { "type": "integer", "minimum": 0, "maximum": 10000 }
      }
    },
    "runtime": {
      "type": "object",
      "properties": {
        "chains":     { "type": "array", "items": { "type": "integer" }, "description": "EIP-155 chain IDs" },
        "model":      { "type": "string", "description": "Underlying LLM or agent framework identifier" },
        "frameworks": { "type": "array", "items": { "type": "string" } }
      }
    }
  }
}
```

### Compatibility Rules

- Consumers MUST ignore unknown top-level and nested fields (forward compatibility).
- Producers MAY omit OPTIONAL fields entirely.
- The `capabilities[].id` namespace is reserved for community-curated canonical tags. Producers MAY use custom tags prefixed with `x-` for experimental capabilities.

### Capability Namespace (Non-Normative)

A recommended initial namespace, to be maintained off-chain:

| Prefix | Meaning | Examples |
|---|---|---|
| `code.*` | Software development | `code.solidity`, `code.rust`, `code.frontend` |
| `audit.*` | Security and quality review | `audit.static`, `audit.semantic` |
| `data.*` | Data processing | `data.etl`, `data.labeling` |
| `trade.*` | Capital operations | `trade.defi.yield`, `trade.mm`, `trade.arbitrage` |
| `content.*` | Generative content | `content.writing`, `content.image` |
| `ops.*` | Operations / infra | `ops.deploy`, `ops.monitoring` |

## Rationale

- **JSON at `tokenURI()` rather than Solidity struct.** Capability taxonomies evolve faster than on-chain storage should. JSON at `tokenURI()` matches ERC-721 metadata precedent and keeps gas costs bounded.
- **Flat capability IDs.** A hierarchical dot-namespaced string (`code.solidity`) enables cheap prefix queries while remaining extensible. Structured capability objects would force every consumer to implement a taxonomy parser.
- **Pricing as a sub-object with a model enum.** The four models (`fixed`, `hourly`, `performance`, `subscription`) cover observed agent commerce patterns. A single `price` number would be insufficient for performance-fee agents that need `performanceFeeBps`.
- **Plural `verification.levels`.** Agents commonly offer multiple verification tiers at different price points; the field is an array, not a single value.
- **`capital-entrusted` as a `jobType`.** Signals compatibility with delegated-execution vaults (see the companion Capital-Entrusted Agent Vault EIP). Clients routing capital jobs can filter on this.
- **Unknown-field tolerance.** The schema will grow; strict validation would break older agents and retard adoption.
- **No on-chain trust of claimed capabilities.** The schema is self-declared. Consumers that need proof rely on reputation, stake, or externally verified attestations — not on the metadata itself.

## Backwards Compatibility

Fully backward compatible with ERC-721 and ERC-8004. Existing Agent NFTs that do not emit compliant metadata remain valid under ERC-8004; this EIP adds only a *conformance profile* for the `tokenURI()` payload. Contracts MAY advertise compliance via ERC-165 on a marker interface (out of scope for this EIP).

## Reference Implementation

Reference JSON instance:

```json
{
  "name": "CodeWriter-v3",
  "version": "3.1.0",
  "description": "Smart-contract developer agent specializing in DeFi primitives",
  "capabilities": [
    { "id": "code.solidity", "level": "expert", "tags": ["defi", "erc20", "erc721"] },
    { "id": "code.rust",     "level": "proficient" },
    { "id": "audit.static",  "level": "proficient" }
  ],
  "pricing": {
    "currency": "USDC",
    "model":    "fixed",
    "min":      "50",
    "max":      "10000"
  },
  "verification": {
    "levels":         ["zkvm", "tee+zkvm"],
    "zkvm":           "sp1",
    "teeAttestation": "0x..."
  },
  "jobTypes": ["one-shot"],
  "sla": {
    "maxConcurrentJobs": 5,
    "responseTimeSec":   3600,
    "availabilityBps":   9900
  },
  "runtime": {
    "chains":     [8453, 56, 1],
    "model":      "claude-opus-4-6",
    "frameworks": ["aacp-sdk", "eliza"]
  }
}
```

## Security Considerations

- **Self-declared data is untrusted.** Metadata is produced by the agent's operator. Consumers MUST NOT treat `capabilities` or `verification.teeAttestation` as proof; TEE attestations MUST be independently verified against hardware roots of trust, and capability claims SHOULD be cross-checked against reputation or work history.
- **URI availability.** `tokenURI()` MAY resolve to centralized hosting that can disappear. Consumers SHOULD prefer content-addressed URIs (IPFS, Arweave) and SHOULD cache metadata.
- **Schema drift.** Older agents may emit earlier schema versions. Consumers MUST handle missing OPTIONAL fields gracefully and MUST NOT hard-fail on unknown fields.
- **Injection in free-text fields.** UIs rendering `name`, `description`, or tag strings MUST sanitize to prevent XSS or terminal-control injection. Marketplaces SHOULD enforce length and character-set limits at ingestion.
- **Privacy.** `operator`, `endpoints`, and `runtime.model` may reveal operator infrastructure. Operators SHOULD treat metadata as public.
- **Griefing via large metadata.** A bloated JSON document can impose indexing cost. Consumers SHOULD cap the metadata size they will parse (RECOMMENDED ≤ 32 KiB).

## Copyright

Copyright and related rights waived via [CC0](../LICENSE).
