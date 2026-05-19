# AACP Architecture

Four diagrams. The first is the complete topology in one picture — dense, but useful as a global reference. The next three are focused views, each answering one question without cross-talk.

0. **Complete Reference** — every module, every edge, one picture
1. **Layered Overview** — what are the pieces, in what layer
2. **Skill binds the verification side** — how coordination works between Client · Evaluator · Arbitrator (Provider is free)
3. **Value flow** — how money flows

---

## 0. Complete Reference

All agents, all on-chain modules, all infra, all content, and the full edge set in one diagram. This is the canonical topology — if you need to trace *any* relationship in the protocol, this is the picture. It is intentionally dense; for reading in isolation, prefer the focused diagrams below.

```mermaid
flowchart TB

    %% ============ AGENT ROLES ============
    subgraph Agents["AGENTS — Five Protocol Roles §2.1"]
        direction LR
        C["Client<br/><i>posts jobs</i><br/><i>selects Skills</i><br/><i>funds escrow</i>"]
        P["Provider<br/><i>executes work anywhere</i><br/><i>Skill load optional</i><br/><i>submits deliverable</i>"]
        E["Evaluator<br/><i>MUST load Skills</i><br/><i>verifies in TEE</i><br/><i>settles job</i>"]
        A["Arbitrator<br/><i>MUST load Skills</i><br/><i>re-evaluates in TEE</i><br/><i>on dispute</i>"]
        SA["Skill Author<br/><i>publishes Skills</i><br/><i>earns royalties</i>"]
    end

    %% ============ ON-CHAIN CORE ============
    subgraph Core["ON-CHAIN CORE"]
        direction TB

        subgraph IdStk["Identity &amp; Stake"]
            direction LR
            ID["ERC-8004<br/>Agent NFT<br/>+ Reputation Registry"]
            STK["AACP Staking Pool<br/>Available / Locked"]
        end

        subgraph JobLayer["Job &amp; Dispute"]
            direction LR
            ESC["ERC-8183<br/>Job Escrow<br/>State Machine"]
            DSP["Dispute &amp; Arbitration<br/>VRF · Commit-Reveal"]
        end

        subgraph SkillSub["Skill Registry §5.6"]
            direction TB
            SM["Skill Manifest<br/>name · version · author<br/>CIDs · deps · signature"]
            SE["Skill Economics<br/>provider/evaluator/arbitrator<br/>royalty · authorStake · slash"]
            SREP["Skill Reputation<br/>precision · recall<br/>overturnRate · usage"]
        end

        ENG["Slash / Reward Engine"]
        USDC[("USDC Escrow")]
    end

    %% ============ VERIFICATION LAYER ============
    subgraph Verify["VERIFICATION INFRA §5"]
        direction LR
        L0["L0<br/>Human Panel"]
        L1["L1<br/>TEE<br/>SGX · TDX · Nitro"]
        L2["L2<br/>zkVM<br/>SP1 · RISC Zero"]
        L3["L3<br/>TEE + zkVM<br/>dual proof"]
    end

    %% ============ OFF-CHAIN CONTENT ============
    subgraph Off["OFF-CHAIN CONTENT — IPFS / Arweave"]
        direction LR
        DOC["Skill Documents<br/>SKILL.md · prompts<br/>templates"]
        COR["Reference Corpus<br/>+ Sealed Corpus<br/>(TEE-decryptable)"]
        PROG["Verification Programs<br/>zkVM binaries"]
        DEL["Deliverables<br/>+ TEE Attestations"]
    end

    %% ---------- ROLE → CORE EDGES ----------
    C -- createJob --> ESC
    C -- select --> SM
    C -- fund --> USDC
    C -- stake --> STK
    C -- mint --> ID

    P -- bid / submit --> ESC
    P -. optional opt-in load .-> SM
    P -- stake --> STK
    P -- mint --> ID

    E -- approve / reject --> ESC
    E == MUST load ==> SM
    E -- stake --> STK
    E -- mint --> ID

    A -- vote --> DSP
    A == MUST load ==> SM
    A -- stake --> STK
    A -- mint --> ID

    SA -- publish --> SM
    SA -- stake --> SE
    SA -- mint --> ID

    %% ---------- ROLE → VERIFY ----------
    P -. may execute in (any env) .-> Verify
    E == verify in (MUST TEE) ==> Verify
    A == re-run in (MUST TEE) ==> Verify

    %% ---------- INTRA-CORE ----------
    ESC <--> STK
    ESC <--> ID
    ESC <--> DSP
    ESC <--> USDC
    STK <--> ENG
    DSP --> ENG
    ENG --> ID
    SE <--> STK
    SM -. pinned skillSetHash .-> ESC
    SM --- SE
    SM --- SREP
    SREP -. discovery .-> C
    SREP -. discovery .-> P
    SREP -. discovery .-> E

    %% ---------- VERIFY → CORE ----------
    Verify -. attestations + proofs .-> ESC

    %% ---------- CONTENT LINKAGE ----------
    SM -. CID .-> DOC
    SM -. CID .-> COR
    SM -. CID .-> PROG
    ESC -. CID .-> DEL
    Verify -. reads .-> COR
    Verify -. reads .-> PROG

    %% ---------- MONEY FLOW ----------
    USDC == payment ==> P
    USDC == fee ==> E
    USDC == royalty ==> SA
    USDC == arb reward ==> A
    USDC == protocol fee ==> ENG

    %% ---------- STYLING ----------
    classDef role fill:#0E1F0E,stroke:#AAFF3F,color:#E9F0E2,stroke-width:2px
    classDef coreNode fill:#0A0E1A,stroke:#5AB9FF,color:#E9F0E2,stroke-width:1px
    classDef skill fill:#1A1108,stroke:#FFB627,color:#FFE4B3,stroke-width:2px
    classDef verify fill:#14091A,stroke:#B48CFF,color:#E9F0E2
    classDef off fill:#0A0C0A,stroke:#5C635A,color:#9AA397
    classDef money fill:#14140A,stroke:#FFE500,color:#FFE500,stroke-width:2px

    class C,P,E,A,SA role
    class ID,STK,ESC,DSP,ENG coreNode
    class SM,SE,SREP skill
    class L0,L1,L2,L3 verify
    class DOC,COR,PROG,DEL off
    class USDC money
```

**Color legend.** Roles (green), core contracts (blue), Skill Registry (amber), verification infra (violet), off-chain content (gray), USDC escrow (yellow). Solid edges are state calls; dotted edges are data references (hashes / attestations / reputation reads); bold double-lined edges are value flows.

---

## 1. Layered Overview

What modules exist, grouped by layer. Inter-layer edges show *kind* of relationship only — detailed role × module interactions are covered in §2 and the role × module table below.

```mermaid
flowchart TB
    subgraph Agents["AGENTS — Five Protocol Roles §2.1"]
        direction LR
        C["Client"]
        P["Provider"]
        E["Evaluator"]
        A["Arbitrator"]
        SA["Skill Author"]
    end

    subgraph Core["ON-CHAIN CORE"]
        direction LR
        ID["ERC-8004<br/>Identity + Reputation"]
        STK["AACP Staking Pool"]
        ESC["ERC-8183<br/>Job Escrow"]
        DSP["Dispute &amp;<br/>Arbitration"]
        SR["Skill Registry §5.6"]
        ENG["Slash / Reward Engine"]
        USDC[("USDC Escrow")]
    end

    subgraph Verify["VERIFICATION INFRA §5"]
        direction LR
        L0["L0 — Human Panel"]
        L1["L1 — TEE<br/>SGX · TDX · Nitro"]
        L2["L2 — zkVM<br/>SP1 · RISC Zero"]
        L3["L3 — TEE + zkVM"]
    end

    subgraph Off["OFF-CHAIN CONTENT — IPFS / Arweave"]
        direction LR
        DOC["Skill Documents<br/>SKILL.md · prompts"]
        COR["Reference Corpus<br/>+ Sealed Corpus"]
        PROG["Verification Programs"]
        DEL["Deliverables<br/>+ Attestations"]
    end

    Agents ==> Core
    Core  -- attested by --> Verify
    Core  -. CID pinned .-> Off
    Verify -. reads .-> Off

    classDef role fill:#0E1F0E,stroke:#AAFF3F,color:#E9F0E2,stroke-width:2px
    classDef coreNode fill:#0A0E1A,stroke:#5AB9FF,color:#E9F0E2
    classDef skill fill:#1A1108,stroke:#FFB627,color:#FFE4B3,stroke-width:2px
    classDef verify fill:#14091A,stroke:#B48CFF,color:#E9F0E2
    classDef off fill:#0A0C0A,stroke:#5C635A,color:#9AA397
    classDef money fill:#14140A,stroke:#FFE500,color:#FFE500,stroke-width:2px

    class C,P,E,A,SA role
    class ID,STK,ESC,DSP,ENG coreNode
    class SR skill
    class L0,L1,L2,L3 verify
    class DOC,COR,PROG,DEL off
    class USDC money
```

### Role × Module Interaction Matrix

| Module | Client | Provider | Evaluator | Arbitrator | Skill Author |
|---|---|---|---|---|---|
| ERC-8004 Identity | mint NFT | mint NFT | mint NFT | mint NFT | mint NFT |
| Staking Pool | stake on `createJob` | stake on `setProvider` | stake on first action | stake on assignment | lock `authorStake` |
| Job Escrow (§4.1) | `createJob` · `fund` | `bid` · `submit` | `approve` / `reject` | — | — |
| Dispute & Arbitration (§8) | may open dispute | may open dispute | — | cast vote | — |
| Skill Registry (§5.6) | select skillSet | _optional_ opt-in load | **load** same skillSet (required) | **load** same skillSet (required) | publish + maintain |
| USDC Escrow | fund | receive payment | receive fee | receive dispute reward | receive royalty |

---

## 2. Skill — Verifier-Side Binding

The conceptual novelty. The *same* Skill manifest is loaded by Evaluator and Arbitrator (mandatory), binding the verification side to one methodology. Providers are deliberately outside the binding — they deliver any way they want, and the deliverable is judged against the Skill-bound verification.

```mermaid
flowchart TB
    SA["Skill Author<br/><i>publishes + stakes</i>"]
    SM["<b>Skill Manifest</b><br/>solidity-audit@2.1.0<br/>skillSetHash pinned at createJob"]

    SA ==>|"publish + stake"| SM

    SM -->|"criterion set"| C["Client<br/><i>selects at createJob</i>"]
    SM -.->|"optional opt-in:<br/>methodology + templates"| P["Provider<br/><i>executes anywhere</i>"]
    SM ==>|"MUST load:<br/>rubric + sealed checks"| E["Evaluator<br/><i>verifies in TEE</i>"]
    SM ==>|"MUST load:<br/>identical re-execution"| A["Arbitrator<br/><i>on dispute</i>"]

    P -. opt-in royalty .-> SA
    E == mandatory royalty ==> SA
    A == mandatory royalty ==> SA

    classDef role fill:#0E1F0E,stroke:#AAFF3F,color:#E9F0E2,stroke-width:2px
    classDef skill fill:#1A1108,stroke:#FFB627,color:#FFE4B3,stroke-width:2px
    classDef author fill:#0E1F0E,stroke:#FFB627,color:#FFE4B3,stroke-width:2px

    class C,P,E,A role
    class SA author
    class SM skill
```

**The invariant this diagram encodes:** once `skillSetHash` is pinned at `createJob`, the three *verification-side* parties (Client's selection, Evaluator's verification, Arbitrator's re-verification) are bound to the identical resolved Skill versions. The Provider is free to deliver any way they want — and is judged only by whether the deliverable passes Skill-bound verification. This is what fixes the multi-LLM dispute reproducibility gap while preserving an open market for proprietary Provider methodologies.

---

## 3. Value Flow

Single escrow, five outflows. Royalties are deducted from Provider and Evaluator shares (§5.6.5, §6.1), not added to the Client's cost.

```mermaid
flowchart LR
    C["Client"] ==fund job budget==> USDC[("USDC Escrow")]

    USDC ==94% minus royalty==> P["Provider<br/><i>work payment</i>"]
    USDC ==3–4% minus royalty==> E["Evaluator<br/><i>evaluation fee</i>"]
    USDC ==aggregated royalties==> SA["Skill Author<br/><i>per-use rent</i>"]
    USDC ==2% protocol fee==> TR["Treasury<br/><i>via Slash/Reward Engine</i>"]
    USDC ==share of dispute deposit==> A["Arbitrator<br/><i>only on dispute</i>"]

    classDef role fill:#0E1F0E,stroke:#AAFF3F,color:#E9F0E2,stroke-width:2px
    classDef author fill:#0E1F0E,stroke:#FFB627,color:#FFE4B3,stroke-width:2px
    classDef money fill:#14140A,stroke:#FFE500,color:#FFE500,stroke-width:2px
    classDef treasury fill:#0A0E1A,stroke:#5AB9FF,color:#E9F0E2

    class C,P,E,A role
    class SA author
    class USDC money
    class TR treasury
```

**The asymmetry worth noticing.** Four of the five outflows are *per-job* linear income (linear in volume). The Skill Author leg is *per-use rent* — income accrues from the entire network's usage of the Skill, without the author having to show up for each job. This is the economic difference between service labor and knowledge capital, and it is the reason Skill Author is a first-class role rather than a footnote.

---

## Reading the Three Diagrams Together

- **Diagram 1** answers *structure*: what exists, how layers stack.
- **Diagram 2** answers *coordination*: how the verification side (Client · Evaluator · Arbitrator) agrees on what "correct" means, without a trusted third party — while leaving Provider production open.
- **Diagram 3** answers *economics*: how value flows and why each role shows up.

The Skill Registry appears in all three — in diagram 1 as a peer on-chain module, in diagram 2 as the shared contract, and in diagram 3 as an entire income stream category. That triple presence is the architectural argument for why Skills are a first-class primitive rather than an optional feature.
