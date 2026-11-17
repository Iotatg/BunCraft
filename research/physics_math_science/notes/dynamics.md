# Dynamics and constraint solving — verified notes

Scope: what the cloned physics repositories actually implement, with
`path:line` evidence, and how it maps onto BunCraft's hair solver. All paths are
relative to `/workspace/BunCraft`. Repositories were **read, not built or
executed**.

## 1. XPBD — the authoritative equations

From `research/upstream/PositionBasedDynamics/PositionBasedDynamics/XPBD.cpp`
(MIT, C++; algorithm reference only):

- Compliance is converted to the time-step-independent form
  `α̃ = 1 / (stiffness · Δt²)` and accumulated into the system diagonal
  (`XPBD.cpp:36-56`).
- The multiplier update is
  `Δλ = −(C + α̃ λ) / (∇C M⁻¹ ∇Cᵀ + α̃)`, followed by positional corrections
  `Δx₀ = +w₀ ∇C Δλ`, `Δx₁ = −w₁ ∇C Δλ` (`XPBD.cpp:36-56`).
- Elastic rods use the same compliance form with a segment-length
  normalisation: `α̃ = (Δt⁻² / K) / averageSegmentLength`, and a stretch
  regularisation of `1e-10 / Δt²`
  (`PositionBasedDynamics/PositionBasedElasticRods.cpp:1188-1202`).

The XPBD paper (Macklin, Müller, Chentanez, NVIDIA, 2016) states the same
update as Eq. (18): `Δλⱼ = (−Cⱼ − α̃ⱼ λᵢⱼ) / (∇Cⱼ M⁻¹ ∇Cⱼᵀ + α̃ⱼ)` with
`α̃ = α / Δt²`. The PDF was fetched with `curl` and converted with `pdftotext`;
`WebFetch` cannot parse PDFs.

## 2. avian — XPBD for joints, impulses for contacts

Version `0.8.0`, `edition = "2024"`
(`research/upstream/avian/crates/avian2d/Cargo.toml:3,4`).

**Important correction to a common assumption:** avian is *not* an XPBD contact
solver. Its own module documentation says so explicitly:

> "Avian currently uses it for joints, while contacts use an impulse-based
> approach."
> — `research/upstream/avian/src/dynamics/solver/xpbd/mod.rs:1-5`

The XPBD solver plugin is feature-gated behind `xpbd_joints`
(`.../solver/mod.rs:14`, `:42`, `:78-79`).

The implementation matches the paper:

```rust
// research/upstream/avian/src/dynamics/solver/xpbd/mod.rs:424-426
let tilde_compliance = compliance / dt.powi(2);
...
(-c - tilde_compliance * lagrange) / (w_sum + tilde_compliance)
```

The same pattern appears for angular constraints
(`.../xpbd/angular_constraint.rs:275,282`) and positional constraints
(`.../xpbd/positional_constraint.rs:98`).

## 3. rapier — rigid bodies, solver, CCD

Version `0.36.1`, `rust-version = "1.86"`, `edition = "2024"`
(`research/upstream/rapier/Cargo.toml:55,67,68`).

Relevant directories under `research/upstream/rapier/src/`:

| Area | Path |
|---|---|
| Velocity / constraint solver | `dynamics/solver/velocity_solver.rs` |
| Staged island solver | `dynamics/solver/staged_island_solver/` |
| Contact constraints | `dynamics/solver/contact_constraint/` |
| Joint constraints | `dynamics/solver/joint_constraint/` |
| Soft bodies / FEM | `dynamics/soft_body/`, `dynamics/solver/soft_fem/` |
| Continuous collision detection | `dynamics/ccd/` |
| Broad phase (BVH) | `geometry/broad_phase_bvh/` |
| Narrow phase | `geometry/narrow_phase/` |
| Island management (sleeping) | `dynamics/island_manager/` |

Rapier is a full rigid-body engine. BunCraft does not need most of it; the
useful ideas are substepping, island-based sleeping, and CCD for fast-moving
tips.

## 4. parry — collision queries

Version `0.31.1`, `edition = "2021"` (`research/upstream/parry/Cargo.toml:11,20`).

| Capability | Evidence |
|---|---|
| GJK closest points | `research/upstream/parry/src/query/gjk/gjk.rs:353` (`pub fn closest_points`) |
| GJK simplex handling | `.../query/gjk/voronoi_simplex2.rs`, `voronoi_simplex3.rs` |
| EPA penetration depth (2D) | `.../query/epa/epa2.rs:103,342` |
| EPA penetration depth (3D) | `.../query/epa/epa3.rs:120,428` |
| SAT | `.../query/sat/` |
| Ray casting / shape casting | `.../query/ray/`, `.../query/shape_cast/` |

## 5. BunCraft's solver

`crates/buncraft-science/src/xpbd.rs` implements, independently:

- `Particle { pos, prev, inv_mass }`
- `DistanceConstraint { a, b, rest, compliance }`
- `solve_distance_xpbd` — `α̃ = compliance/Δt²`,
  `Δλ = (−C − α̃λ)/(w₀+w₁+α̃)`, corrections `+w₀ n Δλ` / `−w₁ n Δλ`.
- `HairStrand` — links plus simplified bend links (`rest = 2·segment`),
  `step(dt, substeps, iterations)` = substepped semi-implicit Euler with
  Gauss–Seidel iterations; λ is reset every substep; velocity is
  `(x − x_prev)/h`.
- `tip()` and `max_stretch()` accessors.

Five unit tests pass: free fall matches semi-implicit Euler, a stiff distance
constraint reaches rest in one solve, compliance limits the correction, a bend
link straightens an L shape, and a hair strand stays attached and stable.

## 6. Hair-strand pseudo-code (not yet wired)

```text
strand = HairStrand::new(root, direction, segments, total_length, mass)
for each frame with fixed dt = 1/120:
    apply gravity  a += (0, 9.81)
    apply wind     a += wind(t) * drag
    for substep in 0..4:
        v += a * (dt/4)
        x_prev = x
        x += v * (dt/4)
        reset lambda
        repeat 8 times:
            solve distance links   (compliance ~1e-4)
            solve bend links       (compliance ~1e-2)
            pin root to scalp anchor (inv_mass = 0)
        v = (x - x_prev) / (dt/4)
    render strand as a tapered stroke through x[0..n]
```

Differences from what exists: the solver is there, but nothing drives it from
`CharacterProfile` or the timeline, and no renderer consumes the resulting
points. This is Step 5 in `science_plan.md`, behind a default-off feature so
the static drawing stays byte-identical.

## 7. Licence position

| Repo | Files | SPDX |
|---|---|---|
| avian | `LICENSE-APACHE`, `LICENSE-MIT` | Apache-2.0 OR MIT |
| rapier | `LICENSE` | Apache-2.0 |
| parry | `LICENSE` | Apache-2.0 |
| PositionBasedDynamics | `LICENSE` | MIT (C++, reference only) |

No GPL/AGPL. No C++ code was copied; the Rust solver is an independent
implementation of the published equations.

## 8. Actionable next steps

1. Keep the solver unwired until there is a pose/timeline consumer; a physics
   solver with no animation is dead code.
2. When wiring, pin the root with `inv_mass = 0` and use 4 substeps at
   1/120 s; the current tests already cover stability at that range.
3. Add a pendulum period test (`2π√(L/g)` within 2 %) before claiming physical
   accuracy in docs.
4. Do not describe the result as "cloth simulation" or "rigid-body physics";
   it is a 1D strand solver.
