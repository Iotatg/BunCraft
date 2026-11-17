# Upstream repository manifest

Cloned with `git clone --depth 1 --single-branch` on 2026-11-15 into
`research/upstream/`. Each SHA below is the exact commit inspected; every
source reference in this study points at that commit. Third-party trees are
kept unmodified and their LICENSE files are preserved.

| Repository | Stars | Language | License (API) | LICENSE file | Commit | Size | Last push |
|---|---:|---|---|---|---|---|---|
| `dimforge/nalgebra` | 4814 | Rust | Apache-2.0 | `LICENSE` | `b2466e6` | 5.1M | 2026-09-30 |
| `sarah-quinones/faer-rs` | 2586 | Rust | MIT | `LICENSE` | `0ecd846` | 13M | 2026-10-22 |
| `linebender/kurbo` | 1028 | Rust | Apache-2.0 | `LICENSE-APACHE`, `LICENSE-MIT` | `a5f97ff` | 1.3M | 2026-10-30 |
| `nical/lyon` | 2614 | Rust | NOASSERTION | `LICENSE-APACHE`, `LICENSE-MIT` | `c3e869f` | 24M | 2026-11-05 |
| `rust-ndarray/ndarray` | 4347 | Rust | Apache-2.0 | `LICENSE-APACHE`, `LICENSE-MIT` | `53df84a` | 2.6M | 2026-10-27 |
| `Ogeon/palette` | 844 | Rust | Apache-2.0 | `LICENSE-APACHE`, `LICENSE-MIT` | `4ba5d65` | 11M | 2026-11-01 |
| `avianphysics/avian` | 3292 | Rust | Apache-2.0 | `LICENSE-APACHE`, `LICENSE-MIT` | `2c2d693` | 16M | 2026-11-04 |
| `dimforge/rapier` | 6073 | Rust | Apache-2.0 | `LICENSE` | `475caa3` | 190M | 2026-11-01 |
| `dimforge/parry` | 892 | Rust | Apache-2.0 | `LICENSE` | `b3c3584` | 5.6M | 2026-10-06 |
| `colour-science/colour` | 2694 | Python | BSD-3-Clause | `LICENSE` | `e412d18` | 17M | 2026-11-13 |
| `InteractiveComputerGraphics/PositionBasedDynamics` | 2294 | C++ | MIT | `LICENSE` | `10a70bc` | 33M | 2026-09-01 |

## Evidence tags

| Tag | Meaning |
|---|---|
| `meta` | GitHub repository metadata (stars, license, language, last push) |
| `readme` | repository README / ARCHITECTURE.md read |
| `source` | source file inspected, with `path:line` |
| `run` | code compiled or executed locally |
| `none` | not verified — do not rely on it |

## Why these eleven

| Area | Repositories | Why BunCraft needs it |
|---|---|---|
| Linear algebra / projection | nalgebra, faer-rs, ndarray | 3/4 view, perspective, least-squares stroke fitting |
| Curve & path geometry | kurbo, lyon | arc-length parametrisation, true Bézier, tessellated stroke outlines |
| Rigid/soft body dynamics | avian, rapier, parry | hair-strand and cloth dynamics, collision, substepping |
| XPBD reference | PositionBasedDynamics | authoritative XPBD / elastic-rod constraint equations (C++, algorithm only) |
| Colour science | palette, colour-science/colour | linear-light blending, OKLab, CIEDE2000, illuminants, transfer functions |

## Licence position

- `Apache-2.0`: nalgebra, kurbo, ndarray, rapier, parry, avian (also MIT), palette (also MIT). Compatible with BunCraft's own MIT-licensed workspace (`Cargo.toml:16`); preserve upstream LICENSE/NOTICE files when vendoring. Apache-2.0 → MIT is a one-way-compatible direction for dependency use.

- `MIT`: faer-rs, PositionBasedDynamics, avian (dual), palette (dual), kurbo (dual), ndarray (dual), lyon (dual).

- `BSD-3-Clause`: colour-science/colour (Python). Algorithm/constant reference only — no code copied into the Rust crates; retain the copyright notice where constants are cited.

- `NOASSERTION` (lyon): repository carries `LICENSE-APACHE` + `LICENSE-MIT` (dual). GitHub's API cannot classify it; the on-disk files are the evidence.

- No GPL/AGPL repository was cloned into this study.

