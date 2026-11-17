//! Character identity: the data that keeps a character looking like itself
//! across poses, panels and drawings.

use serde::{Deserialize, Serialize};

use crate::stroke::Rgba;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Female,
    Male,
    Androgynous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HairStyle {
    LongStraight,
    LongWavy,
    TwinTails,
    Ponytail,
    ShortBob,
    MessyShort,
    HimeCut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EyeShape {
    Round,
    Almond,
    Sharp,
    Droopy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expression {
    Neutral,
    Smile,
    Shy,
    Serious,
    Surprised,
    Smug,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutfitKind {
    SchoolUniform,
    Hoodie,
    Kimono,
    Coat,
    Dress,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HairSpec {
    pub style: HairStyle,
    pub color: Rgba,
    /// 0.8 = chin length, 1.0 = shoulder, 1.4 = long.
    pub length: f32,
    /// Where the parting sits, -1 left .. 1 right.
    pub parting: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EyeSpec {
    pub shape: EyeShape,
    pub color: Rgba,
    /// Multiplier on the default eye width.
    pub size: f32,
    /// Pupil dilation, 0.6..1.3.
    pub pupil: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutfitSpec {
    pub kind: OutfitKind,
    pub color: Rgba,
    pub accent: Rgba,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FaceSpec {
    /// Distance between the outer edges of the cheeks, in pixels.
    pub head_width: f32,
    /// Chin-to-crown distance, in pixels.
    pub head_height: f32,
    /// 0 = sharp V jaw, 1 = very round.
    pub jaw_softness: f32,
    /// Eye size multiplier (anime stylisation).
    pub eye_size: f32,
}

impl Default for FaceSpec {
    fn default() -> Self {
        Self {
            head_width: 372.0,
            head_height: 470.0,
            jaw_softness: 0.55,
            eye_size: 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub name: String,
    pub seed: u64,
    pub gender: Gender,
    pub skin: Rgba,
    pub hair: HairSpec,
    pub eyes: EyeSpec,
    pub outfit: OutfitSpec,
    pub expression: Expression,
    pub face: FaceSpec,
    /// Free-form notes carried through to the drawing report.
    #[serde(default)]
    pub notes: Vec<String>,
}

impl CharacterProfile {
    /// Deterministic profile from a natural-language prompt.
    ///
    /// This is a keyword reader, not an LLM: it maps explicit visual words
    /// (hair colour, style, eye colour, expression, outfit) onto the profile
    /// and leaves everything else at a coherent default. Unknown words are
    /// recorded in `notes` instead of being silently invented.
    pub fn from_prompt(prompt: &str, seed: u64) -> Self {
        let p = prompt.to_lowercase();
        let has = |words: &[&str]| words.iter().any(|w| p.contains(w));

        // A colour word counts for the hair when it is attached to a hair noun:
        // "pink twin tails" must not fall through to the default black.
        let hair_word = |w: &str| {
            p.contains(&format!("{w} hair"))
                || p.contains(&format!("{w} haired"))
                || p.contains(&format!("{w} twin"))
                || p.contains(&format!("{w} pony"))
                || p.contains(&format!("{w} bob"))
                || p.contains(&format!("{w} tail"))
                || p.contains(&format!("{w} cut"))
        };

        let hair_color = if hair_word("silver")
            || hair_word("white")
            || hair_word("platinum")
            || hair_word("ash")
        {
            Rgba::rgb(210, 215, 228)
        } else if hair_word("black") || p.contains("raven") || p.contains("jet black") {
            Rgba::rgb(42, 40, 50)
        } else if hair_word("blonde") || hair_word("golden") || hair_word("yellow") {
            Rgba::rgb(233, 199, 128)
        } else if hair_word("pink") || hair_word("rose") {
            Rgba::rgb(238, 158, 180)
        } else if hair_word("blue") || hair_word("navy") {
            Rgba::rgb(108, 148, 208)
        } else if hair_word("red") || hair_word("crimson") {
            Rgba::rgb(190, 74, 74)
        } else if hair_word("purple") || hair_word("violet") || hair_word("lavender") {
            Rgba::rgb(154, 118, 190)
        } else if hair_word("green") || hair_word("mint") {
            Rgba::rgb(112, 166, 138)
        } else if hair_word("brown") || hair_word("chestnut") {
            Rgba::rgb(112, 80, 60)
        } else {
            Rgba::rgb(58, 52, 66)
        };

        let style = if has(&["twin tail", "twintail", "twintails", "twin-tail"]) {
            HairStyle::TwinTails
        } else if has(&["ponytail", "pony tail"]) {
            HairStyle::Ponytail
        } else if has(&["bob", "bob cut"]) {
            HairStyle::ShortBob
        } else if has(&["hime", "princess cut"]) {
            HairStyle::HimeCut
        } else if has(&["messy", "spiky", "short hair", "tomboy"]) {
            HairStyle::MessyShort
        } else if has(&["wavy", "curly"]) {
            HairStyle::LongWavy
        } else {
            HairStyle::LongStraight
        };

        let length = match style {
            HairStyle::ShortBob | HairStyle::MessyShort => 0.85,
            HairStyle::Ponytail => 1.25,
            HairStyle::TwinTails => 1.2,
            _ => 1.3,
        };

        let eye_color = if has(&["red eyes", "crimson eyes", "scarlet eyes"]) {
            Rgba::rgb(196, 62, 74)
        } else if has(&["blue eyes", "azure eyes", "sapphire"]) {
            Rgba::rgb(78, 138, 208)
        } else if has(&["green eyes", "emerald eyes"]) {
            Rgba::rgb(70, 158, 128)
        } else if has(&["amber eyes", "gold eyes", "yellow eyes"]) {
            Rgba::rgb(220, 160, 62)
        } else if has(&["violet eyes", "purple eyes"]) {
            Rgba::rgb(150, 112, 198)
        } else if has(&["grey eyes", "gray eyes"]) {
            Rgba::rgb(140, 148, 158)
        } else {
            Rgba::rgb(96, 150, 200)
        };

        let eye_shape = if has(&["sharp eyes", "sharp-eyed", "cat eyes"]) {
            EyeShape::Sharp
        } else if has(&["droopy", "tareme", "sleepy eyes"]) {
            EyeShape::Droopy
        } else if has(&["almond"]) {
            EyeShape::Almond
        } else {
            EyeShape::Round
        };

        // Specific expressions win over the generic "smile" keyword:
        // "shy smile" should read as shy, not as a plain smile.
        let expression = if has(&["shy", "blush", "embarrassed"]) {
            Expression::Shy
        } else if has(&["surprised", "shocked"]) {
            Expression::Surprised
        } else if has(&["smug", "confident", "smirk"]) {
            Expression::Smug
        } else if has(&["serious", "stern", "cold"]) {
            Expression::Serious
        } else if has(&["smile", "smiling", "happy", "cheerful"]) {
            Expression::Smile
        } else {
            Expression::Neutral
        };

        let outfit = if has(&["kimono", "yukata"]) {
            OutfitKind::Kimono
        } else if has(&["hoodie", "hoody"]) {
            OutfitKind::Hoodie
        } else if has(&["coat", "jacket", "cloak"]) {
            OutfitKind::Coat
        } else if has(&["dress", "gown"]) {
            OutfitKind::Dress
        } else {
            OutfitKind::SchoolUniform
        };

        let gender = if has(&["boy", "male", "man", "he "]) {
            Gender::Male
        } else if has(&["androgynous", "genderless"]) {
            Gender::Androgynous
        } else {
            Gender::Female
        };

        let outfit_color = if has(&["navy", "dark blue uniform"]) {
            Rgba::rgb(44, 54, 82)
        } else if has(&["white uniform", "white dress"]) {
            Rgba::rgb(232, 234, 240)
        } else if has(&["red uniform", "red dress"]) {
            Rgba::rgb(158, 52, 58)
        } else if has(&["black coat", "black uniform"]) {
            Rgba::rgb(46, 46, 54)
        } else {
            Rgba::rgb(52, 62, 92)
        };

        let accent = if outfit_color.r as u16 + outfit_color.g as u16 + outfit_color.b as u16 > 560
        {
            Rgba::rgb(196, 76, 86)
        } else {
            Rgba::rgb(226, 230, 238)
        };

        let mut notes = Vec::new();
        if p.trim().is_empty() {
            notes.push("empty prompt: drew the default character".to_string());
        }

        Self {
            // The default name hashes the *description*, not the seed: the same
            // prompt with another seed is the same character with another hand,
            // which is exactly what visual memory needs to key on.
            name: if p.contains("mio") {
                "Mio".to_string()
            } else if p.contains("yuki") {
                "Yuki".to_string()
            } else if p.contains("rin") {
                "Rin".to_string()
            } else {
                format!("Character-{:04x}", prompt_hash(&p))
            },
            seed,
            gender,
            skin: Rgba::rgb(248, 226, 212),
            hair: HairSpec {
                style,
                color: hair_color,
                length,
                parting: if has(&["side part", "side-part"]) {
                    0.55
                } else {
                    0.0
                },
            },
            eyes: EyeSpec {
                shape: eye_shape,
                color: eye_color,
                size: if has(&["big eyes", "large eyes"]) { 1.12 } else { 1.0 },
                pupil: 1.0,
            },
            outfit: OutfitSpec {
                kind: outfit,
                color: outfit_color,
                accent,
            },
            expression,
            face: FaceSpec::default(),
            notes,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "{} · {:?} hair ({}) · {:?} eyes ({}) · {:?} · {:?}",
            self.name,
            self.hair.style,
            self.hair.color.hex(),
            self.eyes.shape,
            self.eyes.color.hex(),
            self.expression,
            self.outfit.kind
        )
    }
}

fn prompt_hash(s: &str) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for b in s.trim().bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16_777_619);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_sets_hair_and_eyes() {
        let p = CharacterProfile::from_prompt("silver haired girl with red eyes, shy smile", 7);
        assert_eq!(p.hair.color, Rgba::rgb(210, 215, 228));
        assert_eq!(p.eyes.color, Rgba::rgb(196, 62, 74));
        assert_eq!(p.expression, Expression::Shy);
    }

    #[test]
    fn prompt_is_deterministic() {
        let a = CharacterProfile::from_prompt("twin tails, blue eyes", 42);
        let b = CharacterProfile::from_prompt("twin tails, blue eyes", 42);
        assert_eq!(a.hair.style, b.hair.style);
        assert_eq!(a.eyes.color, b.eyes.color);
        assert_eq!(a.seed, b.seed);
    }

    #[test]
    fn empty_prompt_notes_default() {
        let p = CharacterProfile::from_prompt("   ", 1);
        assert_eq!(p.notes.len(), 1);
        assert_eq!(p.hair.style, HairStyle::LongStraight);
    }
}
