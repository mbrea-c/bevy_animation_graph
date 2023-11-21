use crate::{
    animation::{BoneFrame, BonePose, Pose, PoseFrame, ValueFrame},
    interpolation::linear::InterpolateLinear,
};
use bevy::prelude::*;

pub trait SampleLinear {
    type Output;
    fn sample_linear(&self, time: f32) -> Self::Output;
}

impl<T: InterpolateLinear + FromReflect + TypePath> SampleLinear for ValueFrame<T> {
    type Output = T;

    fn sample_linear(&self, time: f32) -> Self::Output {
        let time = time.clamp(self.prev_timestamp, self.next_timestamp);
        let f = if self.prev_timestamp == self.next_timestamp {
            0.
        } else {
            (time - self.prev_timestamp) / (self.next_timestamp - self.prev_timestamp)
        };

        self.prev.interpolate_linear(&self.next, f)
    }
}

impl SampleLinear for BoneFrame {
    type Output = BonePose;

    fn sample_linear(&self, time: f32) -> Self::Output {
        BonePose {
            rotation: self.rotation.as_ref().map(|v| v.sample_linear(time)),
            translation: self.translation.as_ref().map(|v| v.sample_linear(time)),
            scale: self.scale.as_ref().map(|v| v.sample_linear(time)),
            weights: self.weights.as_ref().map(|v| v.sample_linear(time)),
        }
    }
}

impl SampleLinear for PoseFrame {
    type Output = Pose;

    fn sample_linear(&self, time: f32) -> Self::Output {
        Pose {
            paths: self.paths.clone(),
            bones: self.bones.iter().map(|b| b.sample_linear(time)).collect(),
        }
    }
}
