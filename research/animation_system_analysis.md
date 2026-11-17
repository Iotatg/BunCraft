# Animation system analysis

## OpenToonz MCP

`konkomaji/opentoonz-mcp` (MIT) exposes 59 tools over a real OpenToonz install:
scenes, vector levels, ink strokes, paint, xsheet timing, keyframes, physics,
rigs, camera, fx and render. Its README claims 196 tests, 57 of which drive the
real application and compare pixels. That is the strongest verification story
of any repository in this study, but it depends on OpenToonz being installed and
on a bridge plugin for live editing.

What BunCraft can learn: the unit of animation is an **editable vector level
plus xsheet timing**, not a baked frame. BunCraft's document already stores
actions with `start_ms`; a level is simply a filtered view of actions by layer.

## Whiteboard video engine

Traces SVG/raster lineart into strokes, orders them (large contours before
details), follows a hand gesture, supports word-level timing, renders MP4. Its
README is candid that it is a rendering pipeline, not a drawing intelligence.

What BunCraft can learn: path ordering matters for the *illusion* of drawing.
BunCraft's stage order is a hand-authored version of the same idea.

## Spiralist

Browser-only: photo → one continuous line → filmed drawing. The interesting
part is the single-line constraint: a TSP-like path that never lifts the pen.
BunCraft does not implement this yet; it is a natural future export mode
(`--single-line`), and `plottter` documents a TSP single-line generator.

## BunCraft's animation status

Implemented:

- per-action `Timing`, derived from path length and stage speed;
- `render(up_to_ms)` for frame-accurate replay;
- `--frames N` PNG sequence export;
- browser player with scrub, speed and stage jumps.

Not implemented:

- keyframes and in-betweens;
- camera moves;
- MP4/GIF encoding (would need an encoder dependency);
- manga panel layout and speech bubbles.
