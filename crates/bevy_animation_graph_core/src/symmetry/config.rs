use std::hash::Hash;

use bevy::{
    math::{Isometry3d, Quat, Vec3},
    reflect::{Reflect, std_traits::ReflectDefault},
};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    animation_clip::EntityPath,
    symmetry::serial::{PatternMapperSerial, SymmetryConfigSerial},
};

#[derive(Debug, Default, Reflect, Clone)]
#[reflect(Default)]
pub struct SymmetryConfig {
    pub name_mapper: FlipNameMapper,
    pub mode: SymmertryMode,
}

#[derive(Debug, Reflect, Clone)]
#[reflect(Default)]
pub struct PatternMapper {
    pub key_1: String,
    pub key_2: String,
    pub pattern_before: String,
    pub pattern_after: String,
    #[reflect(ignore, default = "default_regex")]
    pub regex: Regex,
}

impl Hash for PatternMapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key_1.hash(state);
        self.key_2.hash(state);
        self.pattern_before.hash(state);
        self.pattern_after.hash(state);
    }
}

pub fn default_regex() -> Regex {
    Regex::new("").unwrap()
}

impl Default for PatternMapper {
    fn default() -> Self {
        PatternMapperSerial::default().to_value().unwrap()
    }
}

impl Serialize for SymmetryConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SymmetryConfigSerial::from_value(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SymmetryConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SymmetryConfigSerial::deserialize(deserializer).map(|r| r.to_value().unwrap())
    }
}

impl PatternMapper {
    pub fn flip(&self, input: &str) -> Option<String> {
        if let Some(captures) = self.regex.captures(input) {
            let key_capture = captures.get(2).unwrap().as_str();
            let replacement_key = if key_capture == self.key_1 {
                &self.key_2
            } else {
                &self.key_1
            };
            Some(
                self.regex
                    .replace(input, format!("${{1}}{replacement_key}${{3}}"))
                    .into(),
            )
        } else {
            None
        }
    }
}

#[derive(Debug, Reflect, Clone)]
#[reflect(Default)]
pub enum FlipNameMapper {
    Pattern(PatternMapper),
}

#[derive(Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize)]
#[reflect(Default)]
pub enum SymmertryMode {
    /// Mirror about the plane perpendicular to the X axis.
    #[default]
    MirrorX,
}

impl Default for FlipNameMapper {
    fn default() -> Self {
        Self::Pattern(PatternMapper::default())
    }
}

impl FlipNameMapper {
    pub fn flip(&self, input: &EntityPath) -> EntityPath {
        EntityPath {
            parts: input
                .parts
                .iter()
                .map(|part| {
                    let mut part = part.to_string();

                    if let Some(mapped) = match self {
                        Self::Pattern(pattern) => pattern.flip(&part),
                    } {
                        part = mapped;
                    }
                    part.into()
                })
                .collect(),
        }
    }
}

impl SymmertryMode {
    pub fn apply_position(&self, mut input: Vec3) -> Vec3 {
        input.x *= -1.;
        input
    }

    pub fn apply_quat(&self, mut input: Quat) -> Quat {
        input.x *= -1.;
        input.w *= -1.;
        input = -input;
        debug_assert!(input.is_normalized());
        input
    }

    pub fn apply_isometry_3d(&self, mut isometry: Isometry3d) -> Isometry3d {
        isometry.translation = self.apply_position(isometry.translation.into()).into();
        isometry.rotation = self.apply_quat(isometry.rotation);
        isometry
    }
}
