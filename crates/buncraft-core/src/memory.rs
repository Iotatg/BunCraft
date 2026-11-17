//! Visual memory: persistent character profiles so the same character can be
//! drawn again in another pose, panel or episode with identical features.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::profile::CharacterProfile;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VisualMemory {
    #[serde(default)]
    pub characters: BTreeMap<String, CharacterProfile>,
}

impl VisualMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let p = path.as_ref();
        if !p.exists() {
            return Ok(Self::new());
        }
        let text = std::fs::read_to_string(p)?;
        serde_json::from_str(&text)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(p, text)
    }

    pub fn insert(&mut self, profile: CharacterProfile) {
        self.characters.insert(profile.name.clone(), profile);
    }

    pub fn get(&self, name: &str) -> Option<&CharacterProfile> {
        self.characters.get(name)
    }

    /// Reuse the stored identity but allow the caller to override pose or
    /// expression without losing the character's look.
    pub fn restyle(&self, name: &str, f: impl FnOnce(&mut CharacterProfile)) -> Option<CharacterProfile> {
        let mut p = self.characters.get(name)?.clone();
        f(&mut p);
        Some(p)
    }

    pub fn names(&self) -> Vec<&str> {
        self.characters.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let dir = std::env::temp_dir().join("buncraft-memory-test");
        let path = dir.join("memory.json");
        let _ = std::fs::remove_file(&path);
        let mut m = VisualMemory::new();
        m.insert(CharacterProfile::from_prompt("silver hair, red eyes", 3));
        m.save(&path).unwrap();
        let loaded = VisualMemory::load(&path).unwrap();
        assert_eq!(loaded.characters.len(), 1);
        let name = loaded.names()[0].to_string();
        assert!(loaded.get(&name).is_some());
    }
}
