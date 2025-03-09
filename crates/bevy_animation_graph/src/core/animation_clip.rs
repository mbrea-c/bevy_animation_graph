use bevy::{
    animation::{AnimationCurves, AnimationTargetId},
    asset::{prelude::*, ReflectAsset},
    core::prelude::*,
    reflect::prelude::*,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use super::{event_track::EventTrack, id, skeleton::Skeleton};

/// Interpolation method to use between keyframes.
#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default)]
#[reflect(Default)]
pub enum Interpolation {
    /// Linear interpolation between the two closest keyframes.
    #[default]
    Linear,
    /// Step interpolation, the value of the start keyframe is used.
    Step,
    /// Cubic spline interpolation. The value of the two closest keyframes is used, with the out
    /// tangent of the start keyframe and the in tangent of the end keyframe.
    CubicSpline,
}

/// Path to an entity, with [`Name`]s. Each entity in a path must have a name.
#[derive(Reflect, Clone, Debug, Hash, PartialEq, Eq, Default)]
#[reflect(Default, Serialize, Deserialize)]
pub struct EntityPath {
    /// Parts of the path
    pub parts: Vec<Name>,
}

impl EntityPath {
    /// Produce a new `EntityPath` with the given child entity name appended to the end
    pub fn child(&self, child: impl Into<Name>) -> Self {
        let mut new_path = self.clone();
        new_path.parts.push(child.into());
        new_path
    }

    pub fn parent(&self) -> Option<Self> {
        let mut parent = self.clone();
        if parent.parts.len() > 1 {
            parent.parts.remove(parent.parts.len() - 1);
            Some(parent)
        } else {
            None
        }
    }

    pub fn last(&self) -> Option<Name> {
        self.parts.last().cloned()
    }

    /// Returns a string representation of the path, with '/' as the separator. If any path parts
    /// themselves contain '/', they will be escaped
    pub fn to_slashed_string(&self) -> String {
        let mut escaped_parts = vec![];
        for part in &self.parts {
            escaped_parts.push(part.to_string().replace('\\', "\\\\").replace('/', "\\/"));
        }

        escaped_parts.join("/")
    }

    pub fn from_slashed_string(path: String) -> Self {
        Self {
            parts: (InterpretEscapedString { s: path.chars() })
                .map(Name::new)
                .collect(),
        }
    }

    pub fn id(&self) -> id::BoneId {
        AnimationTargetId::from_names(self.parts.iter()).into()
    }
}

struct InterpretEscapedString<'a> {
    s: std::str::Chars<'a>,
}

impl Iterator for InterpretEscapedString<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut curr_item: Vec<char> = vec![];

        while let Some(c) = self.s.next() {
            match c {
                '\\' => match self.s.next() {
                    None => curr_item.push('\\'), // Trailing backslash just gets returned
                    Some('/') => curr_item.push('/'),
                    Some('\\') => curr_item.push('\\'),
                    // etc.
                    Some(c) => curr_item.extend(vec!['\\', c]), // Erroneous escape sequence just is a literal
                },
                '/' => return Some(curr_item.into_iter().collect()),
                c => curr_item.push(c),
            }
        }

        if curr_item.is_empty() {
            None
        } else {
            Some(curr_item.into_iter().collect())
        }
    }
}

impl From<Vec<String>> for EntityPath {
    fn from(value: Vec<String>) -> Self {
        Self {
            parts: value.into_iter().map(Name::new).collect(),
        }
    }
}

impl From<EntityPath> for Vec<String> {
    fn from(value: EntityPath) -> Self {
        value.parts.into_iter().map(|n| n.to_string()).collect()
    }
}

impl Serialize for EntityPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Vec::<String>::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EntityPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<String>::deserialize(deserializer).map(Self::from)
    }
}

#[derive(Asset, Clone, Debug, Default, Reflect)]
#[reflect(Asset)]
pub struct GraphClip {
    // AnimationCurves are non-reflectable
    #[reflect(ignore)]
    pub(crate) curves: AnimationCurves,
    pub(crate) duration: f32,
    pub(crate) skeleton: Handle<Skeleton>,
    pub(crate) event_tracks: HashMap<String, EventTrack>,
}

impl GraphClip {
    /// [`VariableCurve`]s for each animation target. Indexed by the [`AnimationTargetId`].
    pub fn curves(&self) -> &AnimationCurves {
        &self.curves
    }

    /// Get mutable references of [`VariableCurve`]s for each animation target. Indexed by the [`AnimationTargetId`].
    pub fn curves_mut(&mut self) -> &mut AnimationCurves {
        &mut self.curves
    }

    /// Duration of the clip, represented in seconds.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Set the duration of the clip in seconds.
    pub fn set_duration(&mut self, duration_sec: f32) {
        self.duration = duration_sec;
    }

    pub fn skeleton(&self) -> &Handle<Skeleton> {
        &self.skeleton
    }

    pub fn set_skeleton(&mut self, skeleton: Handle<Skeleton>) {
        self.skeleton = skeleton;
    }

    pub fn from_bevy_clip(
        bevy_clip: bevy::animation::AnimationClip,
        skeleton: Handle<Skeleton>,
        event_tracks: HashMap<String, EventTrack>,
    ) -> Self {
        Self {
            curves: bevy_clip.curves().clone(),
            duration: bevy_clip.duration(),
            skeleton,
            event_tracks,
        }
    }
}

//tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_path_slashed_string_roundtrip() {
        let path = "simple/path/here".to_string();
        let path_rountrip = EntityPath::from_slashed_string(path.clone()).to_slashed_string();
        assert_eq!(path, path_rountrip);

        let path = "simple/patl/here/with escaled\\/part".to_string();
        let path_rountrip = EntityPath::from_slashed_string(path.clone()).to_slashed_string();
        assert_eq!(path, path_rountrip);
    }

    #[test]
    fn from_slashed_string_with_escaped_slashes() {
        let path = "simple/path/here/with escaled\\/part".to_string();
        let entity_path = EntityPath {
            parts: vec![
                Name::new("simple"),
                Name::new("path"),
                Name::new("here"),
                Name::new("with escaled/part"),
            ],
        };

        assert_eq!(entity_path, EntityPath::from_slashed_string(path.clone()));
        assert_eq!(path, entity_path.to_slashed_string());
    }
}
