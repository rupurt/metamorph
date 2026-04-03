# VOYAGE REPORT: Reclassify The Compatibility Matrix For Executable Relayouts

## Voyage Metadata
- **ID:** VFlohn41F
- **Epic:** VFg70aqT7
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Reclassify Same-Format Relayout Into Explicit Capabilities
- **ID:** VFlolixv3
- **Status:** done

#### Summary
Replace the generic `same-format-relayout` placeholder with explicit capability entries for the format pairs that now have a real execution or bundle contract, and stop advertising blanket same-format compatibility where no truthful backend story exists yet.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `find_capability()` exposes explicit relayout or bundle-materialization entries for the promoted format pairs and no longer relies on one generic `from == to` placeholder. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-01] Compatibility, planning, and conversion dispatch continue to resolve through shared registry metadata instead of forked per-surface tables. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFlolixv3/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFlolixv3/EVIDENCE/ac-2.log)

### Surface Local-Only And Metadata Blockers In Compatibility
- **ID:** VFloljYuK
- **Status:** done

#### Summary
Add request-specific blockers so the compatibility surface stays truthful when a backend exists but the current request still cannot run because it is remote, targets a remote destination, or lacks required metadata sidecars.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] Compatibility reports for promoted local backends surface blockers for remote sources or unsupported targets instead of implying those requests are runnable. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-03/AC-01] Compatibility reports for `safetensors -> hf-safetensors` surface missing metadata sidecars or unsupported source shapes before conversion executes. <!-- verify: cargo test --workspace, SRS-03:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFloljYuK/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFloljYuK/EVIDENCE/ac-2.log)

### Prove Compatibility Truth For Promoted And Reclassified Paths
- **ID:** VFlolkHve
- **Status:** done

#### Summary
Lock in the promoted and reclassified matrix with direct compatibility proof so the board can distinguish executable, blocked, and unsupported paths from one another.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] Automated proof covers promoted relayout capabilities and any explicitly reclassified same-format requests that no longer have registry entries. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-02/AC-01] Automated proof shows blocked requests remain distinct from unsupported format pairs in compatibility reporting. <!-- verify: cargo test --workspace, SRS-NFR-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFlolkHve/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFlolkHve/EVIDENCE/ac-2.log)


