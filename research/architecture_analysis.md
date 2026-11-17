# Architecture analysis

Five distinct architectural patterns appear across the studied repositories.
BunCraft borrows the *patterns*, not the code.

## 1. Stroke-5 sequence modelling (SketchRNN / Sketchformer / text-to-sketch)

Representation: a sketch is `[(dx, dy, p1, p2, p3), ...]` where `p1` is pen
down, `p2` ends a stroke, `p3` ends the sketch. A decoder predicts the next
5-vector autoregressively.

- **SketchRNN** (`singleline_models`): LSTM encoder-decoder with a Gaussian
  mixture head. Small, fast, but loses long-range structure.
- **Sketchformer** (`leosampaio/sketchformer`): transformer with a learned
  stroke token dictionary; `d_model=128`, 4 layers, 8 heads, `max_seq_len=200`.
  Better reconstruction of long sequences, but 200 points is short for anime.
- **text-to-sketch**: rebuilds Sketchformer in PyTorch with SDPA attention,
  gradient checkpointing, length-bucketed batching, and a 200-position
  checkpoint extended to 4096 positions with a zero-initialised residual. This
  is the only studied project that explicitly targets *anime* stroke sequences,
  and its own README states text conditioning is **not implemented**.

Lesson for BunCraft: stroke-5 is the right interchange format, but a 200-step
context is inadequate for a full portrait (BunCraft emits ~25k points). Any
neural decoder must operate on stroke-level tokens, not raw points.

## 2. Hierarchical stroke generation (Sketch-HARP)

Sketch-HARP decomposes generation into three stages:

1. predict a **stroke embedding** (which stroke),
2. **anchor** it on the canvas (where),
3. decode a **drawing-action sequence** for that stroke (how).

Configuration in `train.py`: stroke embedding 128, encoder RNN 512, decoder
1024, `max_stroke_num=25`, `max_stroke_len=32`, trained on QuickDraw. The
exposed embeddings allow stroke-level replacement and expansion.

Lesson for BunCraft: this is the correct granularity for an artist brain.
BunCraft's `DrawingPlan` + per-stage stroke emission is a hand-authored version
of the same hierarchy: plan the part, place it, then emit its strokes.

## 3. Path finding and optimisation (DrawingBotV3, plottter)

DrawingBotV3 separates **image preprocessing** (60+ filters), **path finding**
(50+ PFMs in the premium build, 3 in the free one), **path optimisation**
(simplify, merge, filter, sort) and **output** (GCode/HPGL/SVG). plottter has a
similar split with 46 generators and explicit hatching/stippling/TSP passes.

Lesson for BunCraft: keep "what to draw" and "how to move the pen" separate.
BunCraft's renderer is deliberately model-agnostic; a future plotter backend
can consume the same `Stroke` list.

## 4. Stroke replay and timing (doodlewithAI, whiteboard-video-engine)

`doodlewithAI` draws with a pencil model that has wobble, pressure and
overshoot, and its viewer replays strokes. `whiteboard-video-engine` traces
contours, orders paths (large shapes before details), follows a hand gesture,
and renders MP4 with word-level timing.

Lesson for BunCraft: timing is part of the artwork. BunCraft stores
`start_ms`/`duration_ms` per action and derives duration from path length, so a
long line genuinely takes longer.

## 5. Editable vector levels and keyframes (opentoonz-mcp)

OpenToonz MCP exposes 59 tools over scenes, vector levels, xsheet timing,
keyframes, rigs and render. It does not generate drawings itself; it drives a
real animation program.

Lesson for BunCraft: animation should be a *timeline over drawing actions*, not
a separate renderer. BunCraft's document already has the action stream; frame
and keyframe layers can be added without touching the rasteriser.

## What BunCraft adopted

| Pattern | BunCraft implementation |
|---|---|
| stroke-5 interchange | `buncraft-brain::dataset` (import/export) |
| hierarchical plan → place → emit | `DrawingPlan` + `Stage` pipeline |
| path/render separation | `buncraft-core` actions vs `buncraft-render` |
| replay with real timing | `Timing` per action, `up_to_ms` rendering |
| vector-first output | SVG export, PNG as a projection |
