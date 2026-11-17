# CLI

```
buncraft draw [OPTIONS]
buncraft inspect [FILE]
buncraft bench [--runs N]
buncraft stages
```

## draw

```bash
buncraft draw \
  --prompt "silver haired girl with red eyes, shy smile, school uniform" \
  --seed 7 \
  --out assets \
  --speed 4
```

| Flag | Default | Meaning |
|---|---|---|
| `--prompt`, `-p` | empty | character description |
| `--seed`, `-s` | 1 | deterministic seed |
| `--out`, `-o` | `assets` | output directory |
| `--speed` | 1.0 | timeline speed multiplier (4 = timelapse) |
| `--no-corrections` | off | disable the correction pass |
| `--frames N` | off | also write N PNG frames |

Outputs:

```
assets/<slug>.png       final artwork, construction hidden
assets/<slug>.svg       vector export
assets/timeline.json    full action stream + timing (latest)
assets/<slug>-timeline.json  per-artwork copy for replay links
assets/report.json      evaluation, stage table, limitations
assets/preview.html     replay player
assets/frames/*.png     optional frame sequence
```

The final line of stdout is a machine-readable summary of counts and score.

## inspect

```bash
buncraft inspect assets/timeline.json
```

Prints canvas, action counts, stage table with start/end times and the
evaluation score.

## bench

```bash
buncraft bench --runs 5
```

Draws five different prompts, renders each, prints per-run and average timings.

## Prompt vocabulary

| Category | Words understood |
|---|---|
| hair colour | silver, white, platinum, ash, black, raven, blonde, golden, pink, rose, blue, navy, red, crimson, purple, violet, lavender, green, mint, brown, chestnut |
| hair style | twin tails, ponytail, bob, hime, messy, spiky, short, wavy, curly, long |
| eye colour | red, crimson, scarlet, blue, azure, green, emerald, amber, gold, violet, grey |
| eye shape | round, almond, sharp, cat, droopy, sleepy |
| expression | smile, happy, shy, blush, serious, stern, surprised, smug, smirk |
| outfit | school uniform, hoodie, kimono, yukata, coat, jacket, dress |
| palette | navy, white, red, black |

Anything else is ignored and recorded in `report.json` under `notes` rather than
being invented.
