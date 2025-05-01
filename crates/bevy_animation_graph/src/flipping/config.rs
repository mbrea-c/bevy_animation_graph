use std::hash::Hash;

use bevy::reflect::{Reflect, std_traits::ReflectDefault};
use regex::{Regex, escape};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Default)]
pub struct FlipConfigOuter<P: Reflect + Default> {
    pub name_mapper: FlipNameMapper<P>,
}

pub type FlipConfig = FlipConfigOuter<PatternMapper>;
pub type FlipConfigProxy = FlipConfigOuter<PatternMapperSerial>;

#[derive(Debug, Reflect, Clone)]
#[reflect(Default)]
pub struct PatternMapper {
    key_1: String,
    key_2: String,
    pattern_before: String,
    pattern_after: String,
    #[reflect(ignore, default = "default_regex")]
    regex: Regex,
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

impl From<FlipConfig> for FlipConfigProxy {
    fn from(value: FlipConfig) -> Self {
        match value.name_mapper {
            FlipNameMapper::Pattern(pm) => Self {
                name_mapper: FlipNameMapper::Pattern(pm.into()),
            },
        }
    }
}

impl TryFrom<FlipConfigProxy> for FlipConfig {
    type Error = regex::Error;

    fn try_from(value: FlipConfigProxy) -> Result<Self, Self::Error> {
        Ok(match value.name_mapper {
            FlipNameMapper::Pattern(pm) => Self {
                name_mapper: FlipNameMapper::Pattern(pm.try_into()?),
            },
        })
    }
}

impl TryFrom<PatternMapperSerial> for PatternMapper {
    type Error = regex::Error;

    fn try_from(value: PatternMapperSerial) -> Result<Self, Self::Error> {
        let regex = Regex::new(&format!(
            "({})({}|{})({})",
            &value.pattern_before,
            escape(&value.key_1),
            escape(&value.key_2),
            &value.pattern_after,
        ))?;

        Ok(Self {
            key_1: value.key_1,
            key_2: value.key_2,
            pattern_before: value.pattern_before,
            pattern_after: value.pattern_after,
            regex,
        })
    }
}

impl From<PatternMapper> for PatternMapperSerial {
    fn from(value: PatternMapper) -> Self {
        Self {
            key_1: value.key_1,
            key_2: value.key_2,
            pattern_before: value.pattern_before,
            pattern_after: value.pattern_after,
        }
    }
}

impl Default for PatternMapper {
    fn default() -> Self {
        Self::try_from(PatternMapperSerial::default()).unwrap()
    }
}

impl Serialize for PatternMapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        PatternMapperSerial::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PatternMapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        PatternMapperSerial::deserialize(deserializer).map(|r| r.try_into().unwrap())
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

#[derive(Debug, Reflect, Serialize, Deserialize, Clone, Hash)]
pub struct PatternMapperSerial {
    pub key_1: String,
    pub key_2: String,
    pub pattern_before: String,
    pub pattern_after: String,
}

impl Default for PatternMapperSerial {
    fn default() -> Self {
        Self {
            key_1: "L".into(),
            key_2: "R".into(),
            pattern_before: r"^.*".into(),
            pattern_after: r"$".into(),
        }
    }
}

#[derive(Debug, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Default)]
pub enum FlipNameMapper<P: Reflect + Default> {
    Pattern(P),
}

impl<P: Reflect + Default> Default for FlipNameMapper<P> {
    fn default() -> Self {
        Self::Pattern(P::default())
    }
}

impl FlipNameMapper<PatternMapper> {
    pub fn flip(&self, input: &str) -> Option<String> {
        match self {
            Self::Pattern(pattern) => pattern.flip(input),
        }
    }
}
