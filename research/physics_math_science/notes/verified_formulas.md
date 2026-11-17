# Verified formulas (cross-checked by the lead agent)

These are the equations I opened in the cloned source myself, in addition to
the four group reports. Line numbers refer to the commits in
`../repo_manifest.md`. Nothing here is paraphrased from a README.

## 1. XPBD compliance — InteractiveComputerGraphics/PositionBasedDynamics

`PositionBasedDynamics/XPBD.cpp:36-56` (`XPBD::solve_DistanceConstraint`):

```cpp
Real alpha = 0.0;
if (stiffness != 0.0) {
    alpha = static_cast<Real>(1.0) / (stiffness * dt * dt);   // :39
    K += alpha;                                               // :40
}
Real Kinv = 1.0 / K;                                          // :44-48
const Real delta_lambda = -Kinv * (C + alpha * lambda);       // :52
lambda += delta_lambda;                                       // :53
const Vector3r pt = n * delta_lambda;                         // :54
corr0 =  invMass0 * pt;                                       // :55
corr1 = -invMass1 * pt;                                       // :56
```

Here `K = Σ wᵢ |∇Cᵢ|²`, so the update is exactly

> Δλ = −(C + α̃·λ) / (Σ wᵢ|∇Cᵢ|² + α̃),  α̃ = 1/(k·Δt²)

where the library calls `k` "stiffness" (a Young's-modulus-like quantity), so
`α̃ = compliance/Δt²` with `compliance = 1/k`. Setting `α̃ = 0` recovers ordinary
PBD projection `Δpᵢ = −wᵢ C ∇Cᵢ / Σ wⱼ|∇Cⱼ|²`.

Elastic rods (the closest published model to a hair strand):
`PositionBasedDynamics/PositionBasedElasticRods.cpp:1188-1202` computes
`bendingAndTorsionCompliance = inverseTimeStepSize² / K` divided by
`averageSegmentLength`, with a stretch regularisation of `1e-10 / Δt²`
(`:1190-1193`). `PositionBasedElasticRods.h:253,332` explicitly refers to
"alpha-tilde in eq. 17" and "eq. 22" of the underlying paper.

## 2. XPBD compliance — avian (Rust, feature-gated)

`avian/src/dynamics/solver/xpbd/mod.rs:229-236` documents

> Δλ = (−C − α̃·λ) / (Σ w + α̃), `α` is compliance (inverse stiffness), `h` is
> the substep size, `α̃ = α/h²`.

`avian/src/dynamics/solver/xpbd/mod.rs:424-426`:

```rust
let tilde_compliance = compliance / dt.powi(2);
(-c - tilde_compliance * lagrange) / (w_sum + tilde_compliance)
```

**Important correction to a common assumption:** avian is *not* an XPBD engine
for contacts. `avian/src/dynamics/solver/xpbd/mod.rs:1-10` says contacts use an
impulse-based approach and XPBD is used for joints; `mod.rs:33` (solver/mod.rs)
gates the whole XPBD plugin behind `#[cfg(feature = "xpbd_joints")]`, and
`XpbdSolverPlugin` is only added under that feature. So "bevy_xpbd → avian ⇒
XPBD everywhere" is wrong at this commit.

## 3. sRGB transfer — Ogeon/palette

`palette/palette/src/encoding/srgb.rs:131-146`:

```rust
// decode (EOTF), :131-132
if x <= 0.04045 { x / 12.92 }
else { ((x + 0.055) / 1.055).powf(2.4) }
// encode (OETF), :145-146
if x <= 0.0031308 { 12.92 * x }
else { 1.055 * x.powf(1.0/2.4) - 0.055 }
```

BunCraft instead interpolates the encoded 8-bit values directly
(`crates/buncraft-core/src/stroke.rs:91-100`) and composites them in that same
encoded space (`crates/buncraft-render/src/lib.rs:343-347`, `inv = 1 − a`,
`cr*a + dst*inv`). Both are physically wrong: light adds in linear radiance,
not in a gamma-compressed signal.

## 4. Arc length of a cubic Bézier — linebender/kurbo

`kurbo/kurbo/src/param_curve.rs` defines `ParamCurveDeriv::gauss_arclen`:

> arc ≈ ½ Σ wᵢ |C′((xᵢ+1)/2)|

`kurbo/kurbo/src/cubicbez.rs:628-666` implements adaptive Gauss–Legendre:
it estimates the 8/16/24-point error from `est³·2.5e-6`, `est⁶·1.5e-11`,
`est⁹·3.5e-16` times `lp_lc` (control-polygon length minus chord), and if none
meets `accuracy` it subdivides at t=0.5 down to `depth = 20`. `inv_arclen`
(`:673+`) inverts that monotone function by bisection/Newton, with tests at
`:863-906` against the closed-form arc length of a known curve.

BunCraft's `catmull_rom` (`crates/buncraft-core/src/geom.rs:124-143`) samples
uniformly in `t` (10 samples per segment), and `resample` (`:185-213`) spaces
points by *chord* length of that already-sampled polyline. There is no analytic
derivative, no curvature, no inverse arc-length solve — so a fast pen movement
and a slow one get the same number of samples per segment, which is why stroke
speed is not physically meaningful yet.

## 5. What is still not verified

- I did not build any of the 11 upstream crates (`run` tag unused for them).
  Their APIs are read from source only.
- No BRDF / light-transport implementation was found in the two colour
  repositories; Lambert/GGX below are standard textbook formulae, **not**
  claims about those repos.
- `lyon`'s GitHub licence field is `NOASSERTION`; the on-disk dual licence
  files are the only evidence used.
