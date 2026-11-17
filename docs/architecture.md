# Architecture

## Data flow

```
prompt ──► CharacterProfile ──► Geometry ──► Stage pipeline ──► Document
                                                                  │
                        ┌─────────────────────────────────────────┤
                        ▼                                         ▼
                 Renderer (PNG)                              SVG export
                        │
                        ▼
                 Replay timeline (JSON) ──► browser player
```

Nothing in the pipeline is an image until the last step. The document is the
artwork; pixels are a projection of it.

## Crates

| Crate | Responsibility | Must not |
|---|---|---|
| `buncraft-core` | geometry, strokes, actions, layers, timing, profiles, memory | depend on a renderer |
| `buncraft-render` | tiny-skia rasteriser, SVG writer, coverage metric | know about anime or prompts |
| `buncraft-anime` | construction geometry, stage pipeline, correction strokes | rasterise |
| `buncraft-brain` | prompt → profile, evaluation, correction, GRU, dataset I/O | draw pixels |
| `buncraft-cli` | `draw`, `inspect`, `bench`, `stages` | contain drawing logic |
| `buncraft-api` | HTTP endpoints, replay server | duplicate the engine |

## Invariants

1. **Determinism.** Given `(prompt, seed, speed, correct)`, the document is
   byte-identical. All randomness comes from `Rng` seeded by the profile.
2. **No images in the core.** `buncraft-core` cannot open a PNG.
3. **Layers are semantic.** Background 0, construction 1, flat colour 2, shadow
   3, lineart 4, effects 5. Render order is layer id, then action order.
4. **Timing is derived.** `duration = path_length / (brush_speed × stage_factor
   × speed)`, clamped. No hand-tuned per-stroke durations.
5. **Corrections are earned.** `apply_corrections` runs only when the evaluator
   measured an error above tolerance.

## Extension points

- **New brush**: add a `BrushKind` variant and a match arm in `stamp`.
- **New character feature**: add a field to `CharacterProfile`, a landmark to
  `Geometry`, and a stage method.
- **New model**: implement `StrokeModel`; the renderer does not change.
- **New output**: consume `Document.actions`; SVG already does.
