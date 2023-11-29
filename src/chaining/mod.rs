use bevy::prelude::*;

use crate::core::frame::{BoneFrame, PoseFrame, ValueFrame};

pub trait Chainable {
    fn chain(&self, other: &Self, duration_first: f32, duration_second: f32, time: f32) -> Self;
}

impl<T: TypePath + FromReflect + Clone> Chainable for ValueFrame<T> {
    fn chain(&self, other: &Self, duration_first: f32, duration_second: f32, time: f32) -> Self {
        if time > duration_first {
            // First frame is finished, second frame is active
            let mut out_pose = other.clone();
            out_pose.map_ts(|t| t + duration_first);
            if out_pose.next_is_wrapped {
                out_pose.next = self.prev.clone();
                out_pose.next_timestamp = self.prev_timestamp + duration_first + duration_second;
            }
            out_pose
        } else if self.next_is_wrapped {
            // First pose is active, but next pose wraps around
            let out_pose = Self {
                timestamp: self.timestamp,
                prev: self.prev.clone(),
                prev_timestamp: self.prev_timestamp,
                next: other.prev.clone(),
                next_timestamp: other.prev_timestamp + duration_first,
                next_is_wrapped: false,
            };

            out_pose
        } else {
            self.clone()
        }
    }
}

impl<T: TypePath + FromReflect + Clone> Chainable for Option<ValueFrame<T>> {
    fn chain(&self, other: &Self, duration_first: f32, duration_second: f32, time: f32) -> Self {
        match (self, other) {
            (Some(frame_1), Some(frame_2)) => {
                Some(frame_1.chain(frame_2, duration_first, duration_second, time))
            }
            (None, None) => None,
            (None, Some(frame_2)) => {
                let mut out = frame_2.clone();
                out.map_ts(|t| t + duration_first);
                Some(out)
            }
            (Some(frame_1), None) => {
                let mut out = frame_1.clone();

                if out.next_is_wrapped {
                    out.next_timestamp = out.next_timestamp + duration_second;
                }

                Some(out)
            }
        }
    }
}

impl Chainable for BoneFrame {
    fn chain(&self, other: &Self, duration_first: f32, duration_second: f32, time: f32) -> Self {
        Self {
            rotation: self
                .rotation
                .chain(&other.rotation, duration_first, duration_second, time),
            translation: self.translation.chain(
                &other.translation,
                duration_first,
                duration_second,
                time,
            ),
            scale: self
                .scale
                .chain(&other.scale, duration_first, duration_second, time),
            weights: self
                .weights
                .chain(&other.weights, duration_first, duration_second, time),
        }
    }
}

impl Chainable for PoseFrame {
    fn chain(&self, other: &Self, duration_first: f32, duration_second: f32, time: f32) -> Self {
        let mut result = PoseFrame::default();

        for (path, bone_id) in self.paths.iter() {
            let Some(other_bone_id) = other.paths.get(path) else {
                continue;
            };

            result.add_bone(
                self.bones[*bone_id].chain(
                    &other.bones[*other_bone_id],
                    duration_first,
                    duration_second,
                    time,
                ),
                path.clone(),
            );
        }

        result
    }
}
