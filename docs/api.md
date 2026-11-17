# HTTP API

```bash
cargo run --release -p buncraft-api -- 8080
```

| Method | Path | Purpose |
|---|---|---|
| GET | `/` | gallery of generated artworks |
| GET | `/player.html?src=...` | replay viewer |
| GET | `/api/health` | `ok` |
| POST | `/api/draw` | draw a character |
| GET | `/api/artworks` | list artworks |
| GET | `/api/artworks/:id/timeline` | action stream |
| GET | `/api/artworks/:id/png` | raster |
| GET | `/api/artworks/:id/svg` | vector |

## Draw

```bash
curl -s localhost:8080/api/draw \
  -H 'content-type: application/json' \
  -d '{"prompt":"twin tails, pink hair, blue eyes, cheerful, hoodie","seed":12,"speed":2}'
```

Response:

```json
{
  "id": "twin-tails-pink-hair-blue-eyes-cheerful-hoodie-12-1",
  "report": { "strokes": 135, "fills": 20, "duration_ms": 29000, "...": "..." },
  "timeline_url": "/api/artworks/.../timeline",
  "png_url": "/api/artworks/.../png",
  "svg_url": "/api/artworks/.../svg",
  "preview_url": "/player.html?src=/api/artworks/.../timeline"
}
```

`correct: false` disables the correction pass. `speed` is clamped to 0.1–32.

Artworks are held in memory; restarting the server clears them. A persistent
store is deliberately not included in the MVP.
