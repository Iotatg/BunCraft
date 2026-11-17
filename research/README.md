# BunCraft research

This directory records what was actually inspected before and during BunCraft's
construction. It is deliberately conservative: a repository's README is
evidence of *intent*, not of working code. Every claim below is tagged with how
it was verified.

| Tag | Meaning |
|---|---|
| `meta` | GitHub API metadata (language, license, size, last push) |
| `readme` | README fetched and read in full or in the relevant part |
| `source` | source files inspected, with `path:line` |
| `run` | code executed locally |
| `none` | not verified, do not rely on it |

## Verification status

- The **original 13-repository study** (the `*_analysis.md` files in this
  directory, listed in `repository_inventory.md`) was desk research: READMEs,
  papers and metadata were read, but no third-party repository was cloned or
  executed.
- The **physics / maths / science study** (`physics_math_science/`) went one
  step further: **11 upstream repositories were cloned** with
  `git clone --depth 1 --single-branch` into `research/upstream/` on 2026-11-15,
  and their source files were read with exact `path:line` references. Exact
  commits, licences and sizes are in
  `physics_math_science/repo_manifest.md`. The trees are unmodified and their
  LICENSE files are preserved.
- **No upstream repository was compiled or executed.** The `run` tag is unused
  across both studies. Where a claim depends on running third-party code, it is
  marked as unverified. Two of the cloned repositories are not even Rust
  dependencies: `colour-science/colour` (Python, BSD-3-Clause) and
  `PositionBasedDynamics` (C++, MIT) are algorithm / constant references only.
- BunCraft remains pure Rust and MIT-licensed (`Cargo.toml:16`). No GPL/AGPL
  code was cloned, and no upstream source was copied into the Rust crates; the
  `crates/buncraft-science/` implementations are independent, with unit tests.

## Reading order

1. `repository_inventory.md`, then `architecture_analysis.md` — the original
   13-repo survey.
2. `physics_math_science/README.md` — the physics / maths / science study,
   including what is verified, what is textbook-only, and what is still unwired.
3. `research_gaps.md` and `docs/limitations.md` — what BunCraft still cannot do.
