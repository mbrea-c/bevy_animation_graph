use bevy::reflect::{Reflect, std_traits::ReflectDefault};
use regex::{Regex, escape};
use serde::{Deserialize, Serialize};

use crate::core::symmetry::config::{FlipNameMapper, PatternMapper, SymmertryMode, SymmetryConfig};

#[derive(Debug, Default, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Default)]
pub struct SymmetryConfigSerial {
    pub name_mapper: FlipNameMapperSerial,
    #[serde(default)]
    pub mode: SymmertryMode,
}

#[derive(Debug, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Default)]
pub enum FlipNameMapperSerial {
    Pattern(PatternMapperSerial),
}

impl Default for FlipNameMapperSerial {
    fn default() -> Self {
        Self::Pattern(PatternMapperSerial::default())
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

impl PatternMapperSerial {
    pub fn to_value(&self) -> Result<PatternMapper, regex::Error> {
        let regex = Regex::new(&format!(
            "({})({}|{})({})",
            &self.pattern_before,
            escape(&self.key_1),
            escape(&self.key_2),
            &self.pattern_after,
        ))?;

        Ok(PatternMapper {
            key_1: self.key_1.clone(),
            key_2: self.key_2.clone(),
            pattern_before: self.pattern_before.clone(),
            pattern_after: self.pattern_after.clone(),
            regex,
        })
    }

    pub fn from_value(value: &PatternMapper) -> Self {
        Self {
            key_1: value.key_1.clone(),
            key_2: value.key_2.clone(),
            pattern_before: value.pattern_before.clone(),
            pattern_after: value.pattern_after.clone(),
        }
    }
}

impl SymmetryConfigSerial {
    pub fn to_value(&self) -> Result<SymmetryConfig, regex::Error> {
        let name_mapper = match &self.name_mapper {
            FlipNameMapperSerial::Pattern(pattern_mapper_serial) => {
                FlipNameMapper::Pattern(pattern_mapper_serial.to_value()?)
            }
        };

        Ok(SymmetryConfig {
            name_mapper,
            mode: self.mode,
        })
    }

    pub fn from_value(value: &SymmetryConfig) -> Self {
        let name_mapper = match &value.name_mapper {
            FlipNameMapper::Pattern(pattern_mapper) => {
                FlipNameMapperSerial::Pattern(PatternMapperSerial::from_value(pattern_mapper))
            }
        };

        SymmetryConfigSerial {
            name_mapper,
            mode: value.mode,
        }
    }
}
