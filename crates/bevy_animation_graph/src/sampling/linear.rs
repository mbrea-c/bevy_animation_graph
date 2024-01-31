use crate::{
    core::{
        frame::{BoneFrame, BonePoseFrame, InnerPoseFrame, ValueFrame},
        pose::{BonePose, Pose},
    },
    interpolation::linear::InterpolateLinear,
};
use bevy::prelude::*;

pub trait SampleLinearAt {
    type Output;
    fn sample_linear_at(&self, time: f32) -> Self::Output;
}

impl<T: InterpolateLinear + FromReflect + TypePath> SampleLinearAt for ValueFrame<T> {
    type Output = T;

    fn sample_linear_at(&self, time: f32) -> Self::Output {
        let time = time.clamp(
            self.prev_timestamp,
            // In order to prevent a silly crash
            // TODO: A better solution must be found
            self.next_timestamp.max(self.prev_timestamp),
        );
        let f = if self.prev_timestamp == self.next_timestamp {
            0.
        } else {
            (time - self.prev_timestamp) / (self.next_timestamp - self.prev_timestamp)
        };

        self.prev.interpolate_linear(&self.next, f)
    }
}

impl SampleLinearAt for BoneFrame {
    type Output = BonePose;

    fn sample_linear_at(&self, time: f32) -> Self::Output {
        BonePose {
            rotation: self.rotation.as_ref().map(|v| v.sample_linear_at(time)),
            translation: self.translation.as_ref().map(|v| v.sample_linear_at(time)),
            scale: self.scale.as_ref().map(|v| v.sample_linear_at(time)),
            weights: self.weights.as_ref().map(|v| v.sample_linear_at(time)),
        }
    }
}

impl SampleLinearAt for InnerPoseFrame {
    type Output = Pose;

    fn sample_linear_at(&self, time: f32) -> Self::Output {
        Pose {
            paths: self.paths.clone(),
            bones: self
                .bones
                .iter()
                .map(|b| b.sample_linear_at(time))
                .collect(),
        }
    }
}

impl SampleLinearAt for BonePoseFrame {
    type Output = Pose;

    fn sample_linear_at(&self, time: f32) -> Self::Output {
        self.inner_ref().sample_linear_at(time)
    }
}
