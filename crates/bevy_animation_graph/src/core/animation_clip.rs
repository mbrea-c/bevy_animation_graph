use bevy::{
    animation::AnimationTargetId,
    asset::prelude::*,
    core::prelude::*,
    math::prelude::*,
    reflect::prelude::*,
    utils::{hashbrown::HashMap, NoOpHash},
};
use serde::{Deserialize, Serialize};

use super::{id, skeleton::Skeleton};

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
    ///
    /// The representation will depend on the interpolation type of this curve:
    ///
    /// - for `Interpolation::Step` and `Interpolation::Linear`, each keyframe is a single value
    /// - for `Interpolation::CubicSpline`, each keyframe is made of three values for `tangent_in`,
    ///   `keyframe_value` and `tangent_out`
    pub keyframes: Keyframes,
    /// Interpolation method to use between keyframes
    pub interpolation: Interpolation,
}

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
    pub(crate) curves: AnimationCurves,
    pub(crate) duration: f32,
    pub(crate) skeleton: Handle<Skeleton>,
}

/// This is a helper type to "steal" the data from a `bevy_animation::AnimationClip` into our
/// `GraphClip`, since the internal fields of `bevy_animation::AnimationClip` are not public and we
/// need to do a hackery.
struct TempGraphClip {
    curves: AnimationCurves,
    duration: f32,
}

/// A mapping from [`AnimationTargetId`] (e.g. bone in a skinned mesh) to the
/// animation curves.
pub type AnimationCurves = HashMap<AnimationTargetId, Vec<VariableCurve>, NoOpHash>;

impl GraphClip {
    #[inline]
    /// [`VariableCurve`]s for each animation target. Indexed by the [`AnimationTargetId`].
    pub fn curves(&self) -> &AnimationCurves {
        &self.curves
    }

    #[inline]
    /// Get mutable references of [`VariableCurve`]s for each animation target. Indexed by the [`AnimationTargetId`].
    pub fn curves_mut(&mut self) -> &mut AnimationCurves {
        &mut self.curves
    }

    /// Gets the curves for a single animation target.
    ///
    /// Returns `None` if this clip doesn't animate the target.
    #[inline]
    pub fn curves_for_target(
        &self,
        target_id: AnimationTargetId,
    ) -> Option<&'_ Vec<VariableCurve>> {
        self.curves.get(&target_id)
    }

    /// Gets mutable references of the curves for a single animation target.
    ///
    /// Returns `None` if this clip doesn't animate the target.
    #[inline]
    pub fn curves_for_target_mut(
        &mut self,
        target_id: AnimationTargetId,
    ) -> Option<&'_ mut Vec<VariableCurve>> {
        self.curves.get_mut(&target_id)
    }

    /// Duration of the clip, represented in seconds.
    #[inline]
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Set the duration of the clip in seconds.
    #[inline]
    pub fn set_duration(&mut self, duration_sec: f32) {
        self.duration = duration_sec;
    }

    /// Adds a [`VariableCurve`] to an [`AnimationTarget`] named by an
    /// [`AnimationTargetId`].
    ///
    /// If the curve extends beyond the current duration of this clip, this
    /// method lengthens this clip to include the entire time span that the
    /// curve covers.
    pub fn add_curve_to_target(&mut self, target_id: AnimationTargetId, curve: VariableCurve) {
        // Update the duration of the animation by this curve duration if it's longer
        self.duration = self
            .duration
            .max(*curve.keyframe_timestamps.last().unwrap_or(&0.0));
        self.curves.entry(target_id).or_default().push(curve);
    }

    pub fn from_bevy_clip(
        bevy_clip: bevy::animation::AnimationClip,
        skelington: Handle<Skeleton>,
    ) -> Self {
        // HACK: to get the corret type, since bevy's AnimationClip
        // does not expose its internals
        let tmp_clip: TempGraphClip = unsafe { std::mem::transmute(bevy_clip) };
        Self {
            curves: tmp_clip.curves,
            duration: tmp_clip.duration,
            skeleton: skelington,
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
