use crate::{
    animation::{BoneFrame, PoseFrame, ValueFrame},
    sampling::linear::SampleLinear,
};
use bevy::prelude::*;

pub trait InterpolateLinear {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self;
}

/// Linear interpolation between morph weights
impl InterpolateLinear for Vec<f32> {
    fn interpolate_linear(&self, other: &Vec<f32>, f: f32) -> Vec<f32> {
        self.iter()
            .zip(other)
            .map(|(old, new)| (new - old) * f)
            .collect()
    }
}

impl InterpolateLinear for Vec3 {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        self.lerp(*other, f)
    }
}

impl InterpolateLinear for Quat {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        self.slerp(*other, f)
    }
}

impl<T: InterpolateLinear + FromReflect + TypePath> InterpolateLinear for ValueFrame<T> {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        // We discard the "edges"
        // |prev_1 xxxx|prev_2 <----keep---->|next_1 xxxx|next_2

        let prev = if self.prev_timestamp < other.prev_timestamp {
            let inter = self.sample_linear(other.prev_timestamp);
            inter.interpolate_linear(&other.prev, f)
        } else {
            let inter = other.sample_linear(self.prev_timestamp);
            inter.interpolate_linear(&self.prev, f)
        };

        let next = if self.next_timestamp < other.next_timestamp {
            let inter = other.sample_linear(self.next_timestamp);
            inter.interpolate_linear(&self.next, f)
        } else {
            let inter = self.sample_linear(other.next_timestamp);
            inter.interpolate_linear(&other.next, f)
        };

        Self {
            prev,
            prev_timestamp: self.prev_timestamp.max(other.prev_timestamp),
            next,
            next_timestamp: self.next_timestamp.min(other.next_timestamp),
            // TODO: Determine the correct behaviour for "blending" next_is_wrapped
            next_is_wrapped: if self.next_timestamp < other.next_timestamp {
                self.next_is_wrapped
            } else {
                other.next_is_wrapped
            },
        }
    }
}

impl InterpolateLinear for BoneFrame {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        let mut result = Self::default();

        // TODO: Maybe we should blend with rest pose whenever one channel is missing?

        match (&self.rotation, &other.rotation) {
            (Some(a), Some(b)) => {
                result.rotation = Some(a.interpolate_linear(&b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.rotation = Some(b.clone()),
            (Some(a), None) => result.rotation = Some(a.clone()),
        }

        match (&self.translation, &other.translation) {
            (Some(a), Some(b)) => {
                result.translation = Some(a.interpolate_linear(&b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.translation = Some(b.clone()),
            (Some(a), None) => result.translation = Some(a.clone()),
        }

        match (&self.scale, &other.scale) {
            (Some(a), Some(b)) => {
                result.scale = Some(a.interpolate_linear(&b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.scale = Some(b.clone()),
            (Some(a), None) => result.scale = Some(a.clone()),
        }

        match (&self.weights, &other.weights) {
            (Some(a), Some(b)) => {
                result.weights = Some(a.interpolate_linear(&b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.weights = Some(b.clone()),
            (Some(a), None) => result.weights = Some(a.clone()),
        }

        result
    }
}

impl InterpolateLinear for PoseFrame {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        let mut result = PoseFrame::default();

        for (path, bone_id) in self.paths.iter() {
            let Some(other_bone_id) = other.paths.get(path) else {
                continue;
            };

            result.add_bone(
                self.bones[*bone_id].interpolate_linear(&other.bones[*other_bone_id], f),
                path.clone(),
            );
        }

        result
    }
}
