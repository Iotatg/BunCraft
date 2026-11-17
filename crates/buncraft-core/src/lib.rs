//! BunCraft core: geometry, drawing actions, documents, characters.
//!
//! Everything downstream (renderer, anime generators, artist brain, API) speaks
//! this model. There is no raster image in the core: a drawing is a *sequence
//! of actions*, and pixels are only a projection of it.

pub mod document;
pub mod geom;
pub mod memory;
pub mod profile;
pub mod rng;
pub mod stroke;

pub use document::{
    Action, Canvas, DocMeta, Document, Evaluation, FillRegion, Issue, Layer, Timing,
    LAYER_BACKGROUND, LAYER_CONSTRUCTION, LAYER_FLAT, LAYER_FX, LAYER_LINEART, LAYER_SHADOW,
};
pub use geom::{arc, catmull_rom, catmull_rom_closed, ellipse, mirror_x, Bounds, Vec2};
pub use memory::VisualMemory;
pub use profile::{
    CharacterProfile, Expression, EyeShape, EyeSpec, FaceSpec, Gender, HairSpec, HairStyle,
    OutfitKind, OutfitSpec,
};
pub use rng::{noise2, Rng};
pub use stroke::{Brush, BrushKind, Point, Rgba, Stage, Stroke};
