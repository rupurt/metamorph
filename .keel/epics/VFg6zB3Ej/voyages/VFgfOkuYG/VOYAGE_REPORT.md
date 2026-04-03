# VOYAGE REPORT: Wire Real Upload Execution Into Library And CLI

## Voyage Metadata
- **ID:** VFgfOkuYG
- **Epic:** VFg6zB3Ej
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Execute Validated Publish Plans Through The Library Upload Flow
- **ID:** VFgfuCNHL
- **Status:** done

#### Summary
Wire the publish executor substrate into the library `publish()` workflow so a validated local bundle can be carried through a real remote write path when execution is explicitly requested.

#### Acceptance Criteria
- [x] [SRS-01/AC-01] `publish()` executes a validated local bundle through the library-owned publish substrate when `execute` is true for the supported destination path. <!-- verify: cargo test --workspace, SRS-01:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-02/AC-01] Preview behavior and validated local bundle stability remain intact while real remote execution is added. <!-- verify: cargo test --workspace, SRS-NFR-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFgfuCNHL/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFgfuCNHL/EVIDENCE/ac-2.log)

### Render Real Publish Outcomes In Upload
- **ID:** VFgfuCtIb
- **Status:** done

#### Summary
Render the new publish execution truth through `metamorph upload` so operators can see the same plan, completion state, and per-artifact outcome details that the library reports.

#### Acceptance Criteria
- [x] [SRS-02/AC-01] `metamorph upload` preserves preview-only behavior by default and renders the same execution truth as the library when `--execute` is supplied. <!-- verify: cargo test --workspace, SRS-02:start:end, proof: ac-1.log-->
- [x] [SRS-NFR-01/AC-01] Command output distinguishes preview, complete publish, partial publish, and guarded refusal clearly enough for operators to understand when remote mutation occurred. <!-- verify: cargo test --workspace, SRS-NFR-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFgfuCtIb/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFgfuCtIb/EVIDENCE/ac-2.log)

### Guard Remote Publish Execution On Validation Credentials And Destination
- **ID:** VFgfuDPIp
- **Status:** done

#### Summary
Keep remote publish execution explicitly guarded so validation, credentials, and destination preflight failures stop the request before any remote write begins, while the CLI continues to consume the library-owned flow.

#### Acceptance Criteria
- [x] [SRS-03/AC-01] Remote publish execution refuses requests that lack validation, explicit execute intent, credentials, or a supported existing destination before remote mutation begins. <!-- verify: cargo test --workspace, SRS-03:start:end, proof: ac-1.log-->
- [x] [SRS-04/AC-01] CLI upload wiring continues to consume the library-owned publish flow instead of introducing CLI-specific upload policy. <!-- verify: cargo test --workspace, SRS-04:start:end, proof: ac-2.log-->
- [x] [SRS-NFR-03/AC-01] Library and CLI publish behavior stays aligned on prerequisites and outcome reporting while guarded execution is introduced. <!-- verify: cargo test --workspace, SRS-NFR-03:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFgfuDPIp/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFgfuDPIp/EVIDENCE/ac-2.log)


