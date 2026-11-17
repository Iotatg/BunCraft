# License compliance

Snapshot from the GitHub API on 2026-11-15. "No declared license" means the
repository has no detectable license file — **not** that it is public domain.

| Repository | License | Reuse decision |
|---|---|---|
| text-to-sketch | none declared | ideas only; no code copied |
| Sketch-HARP | Apache-2.0 | ideas only; patent grant would allow reuse with notice, not needed |
| sketchformer | MIT | ideas only |
| singleline_models | MIT | ideas only |
| Anime2Sketch | MIT | not used |
| spiralist | NOASSERTION | **no reuse**; license unclear |
| DrawingBotV3 | GPL-3.0 | **no code reuse** (copyleft incompatible with MIT); ideas only |
| plottter | none declared | **no reuse** |
| vector_stroke_engine | none declared | **no reuse** |
| doodlewithAI | MIT | ideas only |
| whiteboard-video-engine | MIT | ideas only |
| opentoonz-mcp | MIT | not integrated |
| rea | MIT | not integrated |

## BunCraft's own license

MIT. All source in this repository was written for BunCraft. No third-party
source file, shader, brush texture, model weight or dataset is bundled.

## Rules applied

1. Algorithms and architectural patterns are not copyrightable; implementations
   are. BunCraft re-implements ideas from scratch.
2. GPL-3.0 code (DrawingBotV3) was read for concepts only. Copying even a small
   function would force BunCraft to GPL.
3. Repositories without a license grant no rights beyond viewing. Their code was
   not copied.
4. Dataset licenses are separate from code licenses; see `dataset_research.md`.

## If you contribute

Do not paste code from a repository whose license is not MIT/Apache-2.0/BSD.
If you port an algorithm, cite the paper, not the source file.
