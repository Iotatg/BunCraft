# Colour and light — verified notes

Scope: what the cloned colour-science repositories actually contain, with
`path:line` evidence, and which lighting maths is textbook rather than upstream
code. All paths are relative to `/workspace/BunCraft`. Repositories were
**read, not built or executed**.

## 1. sRGB transfer function

`research/upstream/palette/palette/src/encoding/srgb.rs`:

- Decode (encoded → linear), `:128-132`:
  `x ≤ 0.04045 ? x/12.92 : ((x+0.055)/1.055)^2.4`.
- Encode (linear → encoded), `:143-146`:
  `x ≤ 0.0031308 ? 12.92x : 1.055·x^(1/2.4) − 0.055`.

This is IEC 61966-2-1. `crates/buncraft-science/src/color.rs:14-29` implements
the same piecewise function independently and tests the round trip to `1e-12`.

## 2. Oklab

`research/upstream/palette/palette/src/oklab.rs`:

| Step | Evidence |
|---|---|
| Linear sRGB → LMS′ (M1) | `:281-292` — coefficients `0.4122214708`, `0.5363325363`, `0.0514459929`, … |
| LMS′ → Oklab (M2) | `:296-300` — `0.2104542553`, `0.7936177850`, `-0.0040720468`, … |
| Oklab → LMS′ (M2⁻¹) | `:309` — `0.3963377774`, `0.2158037573`, … |
| LMS → linear sRGB (M1⁻¹) | `:318-320` — `4.0767416621`, `-3.3077115913`, … |

The coefficients are truncated to 10 decimal places, so a forward/inverse round
trip is accurate to about `1e-7`, not to machine epsilon. BunCraft's tests
assert `1e-7` for white anchors and `1e-6` for the round trip, and the comment
in `crates/buncraft-science/src/color.rs` says why.

## 3. colour-science (Python)

Package `colour-science` 0.4.7, `requires-python = ">=3.13,<3.16"`
(`research/upstream/colour/pyproject.toml:2,3,6`). BSD-3-Clause. This is a
**reference implementation**, not a Rust dependency.

| Capability | Evidence |
|---|---|
| CIEDE2000 | `research/upstream/colour/colour/difference/delta_e.py:666` (`def delta_E_CIE2000`) |
| CIE D-series illuminants (incl. D65) | `research/upstream/colour/colour/colorimetry/illuminants.py:254` (`def sd_CIE_illuminant_D_series`) |
| CMFS datasets | `research/upstream/colour/colour/colorimetry/cmfs.py`, `.../colorimetry/datasets/` |
| Chromatic adaptation | `research/upstream/colour/colour/adaptation/` |

BunCraft uses the CIEDE2000 *idea* only through `delta_e_ok` (Oklab Euclidean
distance), which is simpler and cheaper. It does not claim CIEDE2000 accuracy.

## 4. There is no BRDF in either repository

A search for `brdf`, `cook_torrance`, `fresnel` and `ggx` across
`palette/palette/src/` and `colour/colour/` returns **no matches**. Both
libraries stop at colour appearance and colour difference; neither models
surface reflection.

Therefore the lighting code in `crates/buncraft-science/src/light.rs` is
**textbook / paper maths**, not ported upstream code:

- Lambert: `ρ/π · E · max(0, N·L)`.
- Half-Lambert wrap lighting.
- GGX normal distribution, Smith G, Fresnel–Schlick, Cook–Torrance combination.

This distinction is stated in the crate's module docs and must not be blurred
in any write-up.

## 5. What BunCraft does now

**Encoded-space mixing (legacy):** `crates/buncraft-core/src/stroke.rs:91-100`
interpolates gamma-encoded `u8` channels. A 50 % black→white mix yields `128`.

**Linear-light mixing (new):** `Rgba::mix_linear` (`stroke.rs:101-130`) decodes
both colours, interpolates in linear radiance, and re-encodes. The same mix
yields `187`. Alpha is interpolated linearly because it is coverage, not
radiance.

**Where it is used:** the three additive-light sites in
`crates/buncraft-anime/src/lib.rs` — iris glow, and the two hair-highlight tone
bands. The two absorptive sites (iris shade, hair shadow) keep the encoded
`mix`, because shadow is absorption (albedo × visibility), not added light; a
linear lerp toward black would lighten them.

**Where it is not:** the rasteriser still composites in sRGB.
`crates/buncraft-render/src/lib.rs:343-347` blends bytes by hand, and
`circle_path` (`:352-366`) / `soft_circle` (`:370-400`) delegate to
`tiny-skia`, which blends internally in 8-bit sRGB. Fixing only the hand-written
path would create a seam at `r = 26` (`:312-315`).

## 6. Quantified effect

| Case | Encoded | Linear |
|---|---:|---:|
| 50 % black→white, sRGB byte | 128 | 187 |
| Mid-grey 128, Rec.709 luminance | 0.502 (Rec.601 luma on encoded bytes) | 0.2158 |
| Silver hair (210,215,228) + 80 % white | ~246,247,250 | 247,248,250 |
| Black hair (42,40,50) + 58 % cool tint | ~132,139,164 | ~158,167,196 |

The last row is the visible one: a cool highlight on black hair is much more
legible in linear light, which is why the angel ring reads as light instead of
dust.

## 7. Licence position

| Repo | File | SPDX | Use |
|---|---|---|---|
| palette | `LICENSE-APACHE`, `LICENSE-MIT` | Apache-2.0 OR MIT | read; constants cross-checked |
| colour-science/colour | `LICENSE` | BSD-3-Clause | algorithm reference only (Python) |

No code was copied. Constants are cited, and the implementations in
`buncraft-science` are independent with their own tests.

## 8. MSRV note (palette is the exception)

palette 0.7.7 is `edition = "2021"`, `rust-version = "1.71.0"`
(`research/upstream/palette/palette/Cargo.toml:3,24,26`), so it **could** be a
direct dependency of a Rust 1.80 workspace. The other studied Rust engines
cannot (nalgebra 1.89, kurbo 1.85, faer 1.84, ndarray 1.87, lyon edition 2024,
rapier 1.86, parry edition 2021 but part of the same stack, avian edition 2024).

BunCraft still implements colour maths itself, for one reason: a
zero-dependency crate keeps the science testable in isolation and avoids
pulling a large trait/generic surface into a drawing app. If palette were
added later, it should replace `color.rs` behind the same function signatures.

## 9. Actionable next steps

1. Keep additive mixes in linear light; never move shadow lerps there.
2. Add multiplicative linear shading (`albedo × visibility`) for shadows before
   calling the shading physical.
3. Add an `sRGB` chunk / `color-interpolation="linearRGB"` note to exported
   files only after the compositor is actually linear; claiming it earlier
   would be false.
4. Do not add CIEDE2000 unless a task needs it; ΔE_ok is sufficient for
   "is this tone step visible".
