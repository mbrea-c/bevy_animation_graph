use bevy::prelude::*;

use crate::pose::{BonePose, Pose};

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

impl InterpolateLinear for Transform {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        Transform {
            translation: self.translation.interpolate_linear(&other.translation, f),
            rotation: self.rotation.interpolate_linear(&other.rotation, f),
            scale: self.scale.interpolate_linear(&other.scale, f),
        }
    }
}

impl InterpolateLinear for BonePose {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        let mut result = Self::default();

        // TODO: Maybe we should blend with rest pose whenever one channel is missing?

        match (&self.rotation, &other.rotation) {
            (Some(a), Some(b)) => {
                result.rotation = Some(a.interpolate_linear(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.rotation = Some(*b),
            (Some(a), None) => result.rotation = Some(*a),
        }

        match (&self.translation, &other.translation) {
            (Some(a), Some(b)) => {
                result.translation = Some(a.interpolate_linear(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.translation = Some(*b),
            (Some(a), None) => result.translation = Some(*a),
        }

        match (&self.scale, &other.scale) {
            (Some(a), Some(b)) => {
                result.scale = Some(a.interpolate_linear(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.scale = Some(*b),
            (Some(a), None) => result.scale = Some(*a),
        }

        match (&self.weights, &other.weights) {
            (Some(a), Some(b)) => {
                result.weights = Some(a.interpolate_linear(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.weights = Some(b.clone()),
            (Some(a), None) => result.weights = Some(a.clone()),
        }

        result
    }
}

impl InterpolateLinear for Pose {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        self.combine(other, |l, r| l.interpolate_linear(r, f))
    }
}
