# Model comparison

Only architectures that were documented in the studied repositories are listed.
No model below was trained or benchmarked by BunCraft; the comparison is about
*fit for stroke-based anime drawing*, not measured accuracy.

| Model | Params / config | Data | Strength | Limitation for BunCraft |
|---|---|---|---|---|
| SketchRNN (LSTM + GMM) | 2-layer LSTM, 512 hidden (typical), GMM head | QuickDraw stroke-5 | simple, fast, well understood | forgets long structure; 100–200 steps; doodle-level detail |
| Sketchformer (TF2) | d_model 128, 4 layers, 8 heads, max_seq_len 200 | QuickDraw, 345 classes | better reconstruction/interpolation | sequence length far below a portrait; tokenizer needs a dictionary |
| text-to-sketch Sketchformer (PyTorch) | same family, 200 → 4096 positions via zero-init residual | Danbooru2019 portraits → vectorised lineart | closest to anime; long-context extension | no text conditioning; no released weights in repo |
| Sketch-HARP | stroke emb 128, enc 512, dec 1024, 25 strokes × 32 actions | QuickDraw | stroke-level editing, hierarchical | 25 strokes is a doodle budget; anime needs 100+ |
| BunCraft constructive generator | no learned weights; analytic proportions | none | deterministic, explainable, 130+ strokes, full stage plan | not learned; limited style range |

## Why BunCraft does not ship a neural model yet

1. There is **no public anime stroke-sequence dataset**. text-to-sketch derives
   strokes by vectorising lineart; those are *extracted paths*, not recorded
   pen movements, and the repository says so.
2. QuickDraw is genuine human stroke data, but it is doodle-scale: 25 strokes,
   32 points per stroke. Training on it produces doodles, not portraits.
3. Shipping random weights and calling it AI would violate BunCraft's rules.
   The GRU in `buncraft-brain::model` is real inference code with **no trained
   checkpoint**; the CLI never uses it to draw.

## What would change this

A stroke-level anime corpus with (a) per-stroke segmentation, (b) drawing order,
(c) pressure, and (d) a license that permits training. Until then the honest
architecture is: constructive artist + a model interface ready for weights.
