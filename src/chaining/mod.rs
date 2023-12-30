use bevy::prelude::*;

use crate::core::frame::{BoneFrame, InnerPoseFrame, ValueFrame};

pub trait Chainable {
    fn chain(&self, other: &Self, duration_first: f32, duration_second: f32, time: f32) -> Self;
}

impl<T: TypePath + FromReflect + Clone> Chainable for ValueFrame<T> {
    fn chain(&self, other: &Self, duration_first: f32, duration_second: f32, time: f32) -> Self {
        // Note that self and other are queried at the same (relative) time
        // i.e. self is queried at `time`, whereas other is queried at `time - duration_first`
        // That means that it is possible to have the time query be out of range of timestamps

        if time < duration_first {
            match (self.prev_is_wrapped, self.next_is_wrapped) {
                (true, false) => Self {
                    prev: other.prev.clone(),
                    prev_timestamp: other.prev_timestamp,
                    next: self.next.clone(),
                    next_timestamp: self.next_timestamp,
                    prev_is_wrapped: true,
                    // next_is_wrapped should never be true when prev_is_wrapped is true
                    next_is_wrapped: false,
                },
                (false, true) => Self {
                    prev: self.prev.clone(),
                    prev_timestamp: self.prev_timestamp,
                    next: other.next.clone(),
                    next_timestamp: other.next_timestamp + duration_first,
                    prev_is_wrapped: false,
                    next_is_wrapped: false,
                },
                (false, false) => self.clone(),
                (true, true) => {
                    panic!("prev_is_wrapped and next_is_wrapped should never both be true!")
                }
            }
        } else {
            match (other.prev_is_wrapped, other.next_is_wrapped) {
                (true, false) => Self {
                    prev: self.prev.clone(),
                    prev_timestamp: self.prev_timestamp,
                    next: other.next.clone(),
                    next_timestamp: other.next_timestamp + duration_first,
                    prev_is_wrapped: false,
                    next_is_wrapped: false,
                },
                (false, true) => Self {
                    prev: other.prev.clone(),
                    prev_timestamp: other.prev_timestamp + duration_first,
                    next: self.next.clone(),
                    next_timestamp: self.next_timestamp + duration_second,
                    // prev_is_wrapped should never be true when next_is_wrapped is true
                    prev_is_wrapped: false,
                    next_is_wrapped: true,
                },
                (false, false) => {
                    let mut out = other.clone();
                    out.map_ts(|t| t + duration_first);
                    out
                }
                (true, true) => {
                    panic!("prev_is_wrapped and next_is_wrapped should never both be true!")
                }
            }
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
                    out.next_timestamp += duration_second;
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

impl Chainable for InnerPoseFrame {
    fn chain(&self, other: &Self, duration_first: f32, duration_second: f32, time: f32) -> Self {
        let mut result = InnerPoseFrame::default();

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

        result.timestamp = time;

        result
    }
}
