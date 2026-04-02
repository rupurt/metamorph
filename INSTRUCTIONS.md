# INSTRUCTIONS.md

Procedural instructions for humans and agents working with Metamorph through Keel.

## Downstream Contract

This file is downstream from Keel. Keep the turn-loop shape recognizable, but make the repo-specific work surfaces truthful for Metamorph.

When syncing from a newer Keel version, preserve the `PROJECT-SPECIFIC` block instead of regenerating local instructions from memory.

## The Turn Loop

Use Keel's `Orient -> Inspect -> Pull -> Ship -> Close` loop:

1. Orient: `keel heartbeat`, `keel doctor --status`, `keel next --role manager --explain`
2. Inspect: open `README.md`, `POLICY.md`, `ARCHITECTURE.md`, and the relevant story or voyage
3. Pull: choose one bounded slice that advances a real conversion workflow
4. Ship: implement, test, and record proof while context is fresh
5. Close: update board state and land the sealing commit

## Primary Workflows

### Operator (Implementation)

Focus on evidence-backed delivery.

- Context: `keel next --role operator`, `keel story show <id>`, `README.md`, `ARCHITECTURE.md`
- Action: implement or refine library and CLI behavior, add tests, and record proof
- Constraint: keep library behavior and CLI behavior aligned

### Manager (Planning)

Focus on crisp capability definition.

- Context: `keel next --role manager --explain`, `keel mission show <id>`, `README.md`
- Action: decompose work into explicit source-to-target flows, proof steps, and validation criteria
- Constraint: avoid vague planning items like "support models better"; name the formats, layouts, and operator outcome

### Explorer (Research)

Focus on reducing uncertainty in format semantics and downstream runtime expectations.

- Context: `keel bearing list`, upstream docs or local repo evidence, current library abstractions
- Action: document what is format transport, what is tensor transformation, and where the lossy edges are
- Constraint: do not let assumptions about model formats harden into code without written evidence

## Repo-Specific Turn Surfaces

<!-- BEGIN PROJECT-SPECIFIC -->
- Environment:
  - `direnv allow`
  - `nix develop`
- Core verification:
  - `nix develop -c cargo test --workspace`
  - `nix develop -c cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `nix build .#metamorph`
- Useful local commands:
  - `nix develop -c cargo run -p metamorph-cli -- inspect hf://prism-ml/Bonsai-8B-gguf`
  - `nix develop -c cargo run -p metamorph-cli -- convert --input hf://prism-ml/Bonsai-8B-gguf --output ./tmp/bonsai --to hf-safetensors --allow-lossy --plan-only`
  - `nix develop -c cargo run -p metamorph-cli -- validate ./some/path --format hf-safetensors`
  - `nix develop -c keel turn`
- Proof contract:
  - If you change conversion behavior, add or update tests.
  - If you change the public CLI or library story, update `README.md`.
  - If you change high-level project rules, update the relevant foundational doc in the same patch.
- Review constraints:
  - Stop for human review on licensing, redistribution, destructive artifact operations, or changes that would make uploads happen automatically.
  - Keep planned features labeled as planned until they are actually implemented.
<!-- END PROJECT-SPECIFIC -->

## Hygiene Rules

- Use Keel as the canonical lifecycle surface.
- Prefer command evidence over chat-only assertions.
- Keep the foundational docs honest about current implementation status.
- Update these instructions when the repo workflow changes materially.
