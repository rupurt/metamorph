# VOYAGE REPORT: Stabilize Cache And Validation Reuse Loop

## Voyage Metadata
- **ID:** VFeOTEZi2
- **Epic:** VFeOQzrXV
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Define Deterministic Cache Identity
- **ID:** VFeOyo75T
- **Status:** done

#### Summary
Define the first deterministic cache identity contract for representative local and `hf://` sources so later acquisition, reuse, and publish work can build on a stable local naming scheme.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] The library defines which source attributes participate in cache identity, including source kind, detected format, and revision-equivalent metadata when available. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-02/AC-01] Repeated planning or acquisition requests for the same representative source resolve to the same cache identity in tests or command proof. <!-- verify: cargo test --workspace, SRS-NFR-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFeOyo75T/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFeOyo75T/EVIDENCE/ac-2.log)

### Implement Source Acquisition And Reuse Reporting
- **ID:** VFeP0CgFv
- **Status:** done

#### Summary
Implement the first acquisition or reuse slice so operators can see whether Metamorph reused an existing local artifact, copied a local source into managed storage, or fetched a remote source into the cache.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] The library exposes explicit acquisition or reuse outcomes for representative local and `hf://` inputs instead of hiding cache behavior behind a generic success path. <!-- verify: cargo test --workspace, SRS-02:start, proof: ac-1.log-->
- [x] [SRS-02/AC-02] The CLI reports the resulting local path and whether the source was reused or newly materialized. <!-- verify: cargo test --workspace, SRS-02:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-01] Library and CLI surfaces stay aligned on cache hit, miss, and reuse outcomes. <!-- verify: cargo test --workspace, SRS-NFR-01:start, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFeP0CgFv/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFeP0CgFv/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFeP0CgFv/EVIDENCE/ac-3.log)

### Gate Reusable Bundles With Validation
- **ID:** VFeP1hOSP
- **Status:** done

#### Summary
Make validation the gate for reusable outputs so converted bundles are only treated as cacheable or publishable artifacts after the required Hugging Face-style safetensors layout checks pass.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Validation rejects malformed or incomplete output bundles before Metamorph reports them as reusable artifacts. <!-- verify: cargo test --workspace, SRS-03:start, proof: ac-1.log-->
- [x] [SRS-03/AC-02] Successful validation produces an explicit reusable-output result for the primary bundle contract. <!-- verify: cargo test --workspace, SRS-03:end, proof: ac-2.log-->
- [x] [SRS-NFR-01/AC-02] Validation outcomes remain aligned between the library and CLI reporting surfaces. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFeP1hOSP/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFeP1hOSP/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFeP1hOSP/EVIDENCE/ac-3.log)

### Document Cache And Validation Recovery
- **ID:** VFeP36PdD
- **Status:** done

#### Summary
Document the operator recovery path for cache and validation failures so the CLI, README, and foundational docs explain what to check next when local reuse fails.

#### Acceptance Criteria
- [x] [SRS-04/AC-01] Cache failures describe the likely cause and an actionable next step instead of surfacing only low-level errors. <!-- verify: cargo test --workspace, SRS-04:start, proof: ac-1.log-->
- [x] [SRS-04/AC-02] Validation failures identify the missing or malformed bundle elements and direct the operator toward rerun or repair steps. <!-- verify: cargo test --workspace, SRS-04:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFeP36PdD/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFeP36PdD/EVIDENCE/ac-2.log)


