# Dependency map

## Crate graph

```
buncraft-core        (no rendering, no I/O except memory JSON)
   ├── buncraft-render   (tiny-skia)
   ├── buncraft-anime    (constructive artist)
   ├── buncraft-brain    (artist brain, evaluation, GRU, dataset)
   │      ├── buncraft-cli  (clap)
   │      └── buncraft-api  (axum, tokio)
```

`buncraft-core` deliberately has no image dependency: a drawing is data.
`buncraft-anime` has no renderer dependency: it only emits actions.
`buncraft-brain` depends on `buncraft-render` for one function
(`ink_coverage`), which is the only reason the arrow exists.

## External crates

| Crate | Used for | Why not hand-rolled |
|---|---|---|
| `serde` / `serde_json` | document, report, checkpoints | correctness and stability |
| `tiny-skia` | rasterisation, PNG encode | mature, pure Rust, no C deps |
| `clap` | CLI | standard, low risk |
| `axum` / `tokio` | HTTP API | small, well tested |
| `anyhow` | CLI errors | ergonomics only |

No Python, no Node, no native ML runtime. The engine is 100% Rust; the browser
player is a static viewer for exported timeline JSON and is not part of the
drawing engine.

## Build footprint

`cargo build --release` produces two binaries (`buncraft`, `buncraft-api`) and
no shared libraries beyond libc. No GPU is required; there is no CUDA path yet.
