# Drawing model

## Coordinate system

Canvas pixels, origin top-left, +x right, +y down. A portrait is 1200×1600.
No DPI scaling is stored; `RenderOptions.scale` handles output size.

## Actions

```rust
enum Action {
    Stroke(Stroke),
    Fill(FillRegion),
}
```

A `Stroke` is:

| Field | Meaning |
|---|---|
| `points` | `{x, y, pressure}` polyline, already flattened |
| `brush` | kind, size, opacity, colour, taper, jitter |
| `layer` | semantic layer id |
| `stage` | which phase of the drawing produced it |
| `role` | human-readable intent, e.g. `"left upper eyelid"` |
| `clip` | optional polygon mask |
| `seed` | texture/tremor seed |

A `FillRegion` is a polygon, colour, opacity and optional clip. It is a paint
bucket action, not an image paste.

## Brushes

| Kind | Behaviour |
|---|---|
| `pencil` | grainy, 55% opacity, pressure-sensitive, tremor |
| `ink` | crisp, near-opaque, thin |
| `brush` | strong taper, organic width |
| `marker` | wide, translucent, flat colour |
| `airbrush` | radial-gradient falloff, blush and washes |

## Layers

| Id | Name | Contents |
|---|---|---|
| 0 | background | wash |
| 1 | construction | axes, guides, rough sketch (hidden in the final image) |
| 2 | flat colour | skin, hair, outfit, iris, sclera |
| 3 | shadow | hatching, blush, cast shadow |
| 4 | lineart | ink contour, eyes, hair, clothing |
| 5 | effects | highlights, catchlights |

## Timing

Every action gets `{start_ms, duration_ms}`. Duration comes from path length,
brush base speed and a stage factor (lineart is slower than construction).
`render(up_to_ms)` draws only the fraction of each stroke that would exist at
that moment, so replay is frame-accurate rather than a fade-in.

## Stage pipeline

1. composition — axes, head volume, shoulder line
2. rough sketch — loose head shape, gesture, hair mass
3. construction — face cross, eye boxes, nose/mouth guides, ears
4. facial structure — contour halves, ears, neck
5. eyes — lids, iris, pupil, lash, brow
6. nose & mouth — expression-dependent
7. hair — silhouette, tails, fringe, side locks, strands
8. body — torso, collarbones
9. clothing — collar/hood, ribbon, folds
10. clean lineart — second, confident pass in ink
11. colour — flat fills, clipped iris, ribbon, highlights
12. shading — soft neck and forehead shadow, blush, eyelid shadow
13. correction — only if the evaluator found an error

## Hair

Hair is where a portrait is won or lost, so it is built in four passes rather
than one flat shape:

- **Silhouette.** A closed mass whose width is sampled by
  `hair_half_width_at(y)`. Everything drawn inside the hair (strands, shading,
  highlights) is placed relative to that width, so nothing spills onto a cheek.
- **Fringe.** Drawn in *clumps*: two or three strands leave the parting apart
  and converge on one tapered tip, with bare hair between clumps. Evenly spaced
  parallel lines read as a barcode, which is what the first version looked like.
- **Side strands.** Four curved strands per side mass, following the outline.
  Without them a long hairstyle is one slab of colour.
- **Highlight ("angel ring").** An arch that follows the dome of the skull:
  high at the crown, drooping at the temples. It is **broken into three
  glints** with gaps of bare hair between them — one continuous strip reads as
  a headband, whereas broken light reads as reflection. Each glint tapers to
  zero at both ends and has a shallow serrated edge (6 % amplitude, not a
  zipper). It is drawn under the lineart so strand lines cross it.
- **Highlight tone.** Chosen from the hair's luma in three bands: near-white
  hair goes almost to white (80 % mix), mid-tone hair gets a 42 % white mix so
  the hue survives (pink hair must not turn blue-grey), and dark hair gets a
  cool tint, because a neutral grey band over black reads as dust. The mix is
  performed in **linear light** (`Rgba::mix_linear`), not on gamma-encoded
  bytes: a highlight is added radiance, so the encoded-space lerp was
  systematically too dim.
- **Shadows stay absorptive.** Iris and hair-root shadows still use the encoded
  `Rgba::mix` toward a near-black colour. Shadow is absorption (albedo
  multiplication), not added light; a linear lerp toward black would *lighten*
  them. Multiplicative linear shading is the next step, not a mix.

## JSON

`buncraft draw` writes the whole document as `timeline.json`. It is the input to
the browser player and to `buncraft inspect`. Schema is the serde representation
of `Document`; `kind` discriminates `stroke` and `fill`.
