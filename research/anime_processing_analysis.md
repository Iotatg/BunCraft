# Anime processing analysis

## Two fundamentally different routes

### A. Extract from an existing image

`Anime2Sketch` (MIT, 2.1k stars) is an image-to-lineart network for anime and
illustrations. `text-to-sketch` goes further: download Danbooru2019 portraits,
extract lineart, filter, vectorise centre lines, order the resulting paths, and
export stroke-5. That pipeline is real and documented in its README.

Consequence: the strokes are **derived from pixels**, not recorded from a hand.
The repository is explicit that text conditioning is not implemented, and its
"current state" table is unusually honest about what works.

### B. Construct from anatomy

BunCraft takes the second route. A `CharacterProfile` fixes head width/height,
jaw softness, eye size/shape/colour, hair style/colour/length, expression,
outfit and palette. `Geometry` converts those into landmarks:

| Landmark | Rule used |
|---|---|
| eye line | 57.5% of head height from crown |
| nose base | 73.5% |
| mouth | 84.5% |
| eye centres | ±49.5% of half-width |
| chin | crown + head height |
| shoulders | chin + 168 px (bust framing) |

These are the proportions taught in beginner manga construction, and they are
why the same profile redrawn with a different seed still looks like the same
character.

## What extraction gives that construction does not

- Real artist line quality and stylistic detail.
- Arbitrary poses, hands, clothing folds, perspective.

## What construction gives that extraction does not

- A genuine **drawing order** (composition → rough → construction → features →
  hair → body → clothing → lineart → colour → shading).
- Per-stroke intent (`role`) and a plan to self-check against.
- Reproducibility from a seed, with no model weights.

## BunCraft's honest position

The constructive route is the MVP. It cannot yet match an extracted lineart in
richness. `research_gaps.md` records the missing bridge: a stroke-level anime
corpus and a model that predicts strokes conditioned on a character profile.
