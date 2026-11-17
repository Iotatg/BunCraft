# Dataset research

## Suitable public data

| Dataset | Content | Format | License | Fit |
|---|---|---|---|---|
| Quick, Draw! | 50M doodles, 345 categories | stroke-3 `.npz` | CC BY 4.0 (Google) | genuine human strokes, but doodle-scale |
| QuickDraw stroke-5 (SketchRNN) | simplified, 25 strokes max | `.npz` | CC BY 4.0 | directly usable by `buncraft-brain::dataset` after conversion |
| Danbooru2019 portraits | anime images | images | images remain © their artists; research use only | lineart source, **not** stroke data |
| Anime2Sketch outputs | lineart | images | derived from the above | still images, not sequences |

## What "stroke data" means

A stroke dataset must contain, per drawing:

1. an ordered list of strokes,
2. ordered points within each stroke,
3. pen-up/pen-down state,
4. ideally pressure and timing.

QuickDraw satisfies 1–3. It does **not** contain pressure or timing, and its
drawings are 10–30 second doodles. No public anime dataset satisfies 1–4.

`text-to-sketch` produces stroke-5 by vectorising lineart. Those paths are
geometrically valid but their order is inferred by a tracing algorithm, not
recorded from a human hand. BunCraft labels such data as *extracted*, never as
*recorded*.

## BunCraft's dataset format

JSONL, one sketch per line:

```json
{"label":"face","strokes":[[[10,20],[12,24]],[[30,40],[32,42]]]}
```

`buncraft-brain::dataset` converts it to stroke-5, normalises coordinates and
splits train/validation deterministically. Unit tests cover offsets, pen flags
and blank lines.

## Licensing caution

- QuickDraw: attribution required (CC BY 4.0).
- Danbooru: the images are copyrighted by individual artists; a model trained on
  them inherits legal risk. BunCraft does not bundle any such data.
- Do not redistribute derived lineart without checking the source terms.
