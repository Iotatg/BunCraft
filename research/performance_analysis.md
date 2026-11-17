# Performance analysis

Measured on the build machine (32 vCPU, 123 GiB RAM, no GPU). Numbers are from
`buncraft bench` and `time buncraft draw`.

## Generation

| Metric | Debug | Release |
|---|---|---|
| draw ~140 strokes / ~31k points | 6 ms | 1 ms |
| evaluate + correction | < 1 ms | < 1 ms |
| document JSON size | ~2.5 MB | ~2.5 MB |

Generation is negligible. The interesting cost is rasterisation.

## Rasterisation, 1200×1600

| Build | Time | Notes |
|---|---|---|
| debug | ~13 s | pixel loops unoptimised |
| release | 0.305–0.312 s | `buncraft bench --runs 3` |

The first implementation used one tiny-skia path fill per brush stamp and took
**124 s** for the same drawing. Replacing sub-26px stamps with a direct
premultiplied pixel blend and switching airbrushes to a single radial gradient
cut that by roughly two orders of magnitude.

End-to-end `buncraft draw` (generate + PNG + SVG + JSON + report) takes
**0.35 s** in release on this machine.

## Memory

- Document for one portrait: ~28k points × 12 bytes ≈ 340 KB plus JSON overhead.
- Pixmap 1200×1600 RGBA: 7.7 MB.
- Peak during render: one layer pixmap + output ≈ 16 MB.

## Scaling levers

| Lever | Effect |
|---|---|
| stamp spacing `0.55 × radius` | lower = smoother, linearly slower |
| authoring spacing `0.45 × size` | lower = more points, larger JSON |
| canvas size | quadratic in pixels |
| `--frames N` | N full renders |

## Not measured

- Neural inference (no trained model).
- SVG file size vs complexity (currently ~2.2 MB for a portrait because every
  point becomes a cubic segment).
- Browser playback frame rate (the player is a viewer, not the engine).
