use bevy::{
    asset::prelude::*, core::prelude::*, math::prelude::*, reflect::prelude::*, utils::HashMap,
};
use serde::{Deserialize, Serialize};

/// List of keyframes for one of the attribute of a [`Transform`].
///
/// [`Transform`]: bevy::transform::prelude::Transform
#[derive(Reflect, Clone, Debug)]
pub enum Keyframes {
    /// Keyframes for rotation.
    Rotation(Vec<Quat>),
    /// Keyframes for translation.
    Translation(Vec<Vec3>),
    /// Keyframes for scale.
    Scale(Vec<Vec3>),
    /// Keyframes for morph target weights.
    ///
    /// Note that in `.0`, each contiguous `target_count` values is a single
    /// keyframe representing the weight values at given keyframe.
    ///
    /// This follows the [glTF design].
    ///
    /// [glTF design]: https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#animations
    Weights(Vec<f32>),
}

/// Describes how an attribute of a [`Transform`] or morph weights should be animated.
///
/// `keyframe_timestamps` and `keyframes` should have the same length.
///
/// [`Transform`]: bevy::transform::prelude::Transform
#[derive(Reflect, Clone, Debug)]
pub struct VariableCurve {
    /// Timestamp for each of the keyframes.
    pub keyframe_timestamps: Vec<f32>,
    /// List of the keyframes.
    pub keyframes: Keyframes,
}

/// Path to an entity, with [`Name`]s. Each entity in a path must have a name.
#[derive(Reflect, Clone, Debug, Hash, PartialEq, Eq, Default)]
#[reflect(Default)]
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
}

struct InterpretEscapedString<'a> {
    s: std::str::Chars<'a>,
}

impl<'a> Iterator for InterpretEscapedString<'a> {
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

/// A list of [`VariableCurve`], and the [`EntityPath`] to which they apply.
#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct GraphClip {
    pub(crate) curves: Vec<Vec<VariableCurve>>,
    pub(crate) paths: HashMap<EntityPath, usize>,
    pub(crate) duration: f32,
}

impl GraphClip {
    #[inline]
    /// [`VariableCurve`]s for each bone. Indexed by the bone ID.
    pub fn curves(&self) -> &Vec<Vec<VariableCurve>> {
        &self.curves
    }

    /// Gets the curves for a bone.
    ///
    /// Returns `None` if the bone is invalid.
    #[inline]
    pub fn get_curves(&self, bone_id: usize) -> Option<&'_ Vec<VariableCurve>> {
        self.curves.get(bone_id)
    }

    /// Gets the curves by it's [`EntityPath`].
    ///
    /// Returns `None` if the bone is invalid.
    #[inline]
    pub fn get_curves_by_path(&self, path: &EntityPath) -> Option<&'_ Vec<VariableCurve>> {
        self.paths.get(path).and_then(|id| self.curves.get(*id))
    }

    /// Duration of the clip, represented in seconds
    #[inline]
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Add a [`VariableCurve`] to an [`EntityPath`].
    pub fn add_curve_to_path(&mut self, path: EntityPath, curve: VariableCurve) {
        // Update the duration of the animation by this curve duration if it's longer
        self.duration = self
            .duration
            .max(*curve.keyframe_timestamps.last().unwrap_or(&0.0));
        if let Some(bone_id) = self.paths.get(&path) {
            self.curves[*bone_id].push(curve);
        } else {
            let idx = self.curves.len();
            self.curves.push(vec![curve]);
            self.paths.insert(path, idx);
        }
    }

    /// Whether this animation clip can run on entity with given [`Name`].
    pub fn compatible_with(&self, name: &Name) -> bool {
        self.paths.keys().any(|path| &path.parts[0] == name)
    }
}

impl From<bevy::animation::AnimationClip> for GraphClip {
    fn from(value: bevy::animation::AnimationClip) -> Self {
        // HACK: to get the corret type, since bevy's AnimationClip
        // does not expose its internals
        unsafe { std::mem::transmute(value) }
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
