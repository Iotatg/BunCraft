# Repository inventory

Snapshot date: 2026-11-15. Stars and licenses come from the GitHub API at that
time. "Verified" uses the tags defined in `README.md`.

## AI drawing intelligence

| Repository | Lang | License | Stars | What it actually is | Verified |
|---|---|---|---|---|---|
| [Long-form-AI-video-generation/text-to-sketch](https://github.com/Long-form-AI-video-generation/text-to-sketch) | Python | none declared | 0 | Anime portrait → lineart → centerline vectorisation → ordered strokes → stroke-5 → Sketchformer training | meta, readme |
| [SCZang/Sketch-HARP](https://github.com/SCZang/Sketch-HARP) | Python | Apache-2.0 | 6 | AAAI 2026 hierarchical autoregressive sketch model with stroke-level editing | meta, readme |
| [leosampaio/sketchformer](https://github.com/leosampaio/sketchformer) | Python | MIT | 63 | TensorFlow 2 reference implementation of Sketchformer (CVPR 2020) | meta, readme |
| [andrewlook/singleline_models](https://github.com/andrewlook/singleline_models) | Jupyter | MIT | 5 | PyTorch SketchRNN (LSTM + GMM) and SketchFormer, single-line dataset | meta, readme |

## Anime drawing and lineart

| Repository | Lang | License | Stars | What it actually is | Verified |
|---|---|---|---|---|---|
| [Mukosame/Anime2Sketch](https://github.com/Mukosame/Anime2Sketch) | Python | MIT | 2146 | Image → anime lineart extractor (U-Net style), not a stroke model | meta, readme |
| [winchxyz/spiralist](https://github.com/winchxyz/spiralist) | JavaScript | NOASSERTION | 69 | Browser app: photo → one continuous line → filmed drawing | meta, readme |

## Drawing engines and rendering

| Repository | Lang | License | Stars | What it actually is | Verified |
|---|---|---|---|---|---|
| [SonarSonic/DrawingBotV3](https://github.com/SonarSonic/DrawingBotV3) | Java | GPL-3.0 | 535 | Image → stylised vector plots; 50+ path-finding modules, pen settings, GCode/HPGL | meta, readme |
| [pywkt/plottter](https://github.com/pywkt/plottter) | Python | none declared | 32 | Desktop plotter-art app, 46 generators, SVG/HPGL/G-code | meta, readme |
| [ChallaYogeswar/vector_stroke_engine](https://github.com/ChallaYogeswar/vector_stroke_engine) | TypeScript | none declared | 0 | Browser photo tool: posterise/sketch/ASCII/3D, SVG export, 112 tests | meta, readme |
| [lazyperspective/doodlewithAI](https://github.com/lazyperspective/doodlewithAI) | JavaScript | MIT | 10 | Stroke-by-stroke pencil drawings in plain JS, replay viewer | meta, readme |
| [gnipbao/whiteboard-video-engine](https://github.com/gnipbao/whiteboard-video-engine) | Python | MIT | 116 | SVG/raster → traced strokes → ordered → MP4 whiteboard video | meta, readme |

## Animation

| Repository | Lang | License | Stars | What it actually is | Verified |
|---|---|---|---|---|---|
| [konkomaji/opentoonz-mcp](https://github.com/konkomaji/opentoonz-mcp) | Python | MIT | 4 | MCP server driving OpenToonz: vector levels, xsheet, keyframes, render (59 tools) | meta, readme |

## Research tooling

| Repository | Lang | License | Stars | What it actually is | Verified |
|---|---|---|---|---|---|
| [morluto/rea](https://github.com/morluto/rea) | TypeScript | MIT | 460441* | MCP server + CLI for reverse engineering app behaviour and native binaries | meta, readme |

\* The star count returned by the API is implausibly high for a research tool
and is reported as-is rather than trusted.

## What this inventory does **not** say

- It does not say any of these projects run, train, or reproduce their papers.
- It does not say their checkpoints are usable: Sketch-HARP's weights are on
  Baidu Pan, Sketchformer's are TensorFlow-1/2 era, text-to-sketch ships no
  weights in the repository.
- It does not say the licenses permit reuse. See `license_compliance.md`.
