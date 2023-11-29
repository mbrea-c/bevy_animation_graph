use crate::{
    core::frame::{BoneFrame, PoseFrame, ValueFrame},
    sampling::linear::SampleLinearAt,
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

impl<T: InterpolateLinear + FromReflect + TypePath + std::fmt::Debug + Clone> InterpolateLinear
    for ValueFrame<T>
{
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        // We discard the "edges"
        // |prev_1 xxxx|prev_2 <----keep---->|next_1 xxxx|next_2

        if self.timestamp > self.next_timestamp {
            println!(
                "Yo we skippin? {} --[ {} ]-- {}",
                self.prev_timestamp, self.timestamp, self.next_timestamp
            );
            return other.clone();
        }

        // Then consider overlapping frames
        let prev = if self.prev_timestamp < other.prev_timestamp {
            let inter = self.sample_linear_at(other.prev_timestamp);
            inter.interpolate_linear(&other.prev, f)
        } else {
            let inter = other.sample_linear_at(self.prev_timestamp);
            inter.interpolate_linear(&self.prev, 1. - f)
        };

        let next = if self.next_timestamp < other.next_timestamp {
            let inter = other.sample_linear_at(self.next_timestamp);
            inter.interpolate_linear(&self.next, 1. - f)
        } else {
            let inter = self.sample_linear_at(other.next_timestamp);
            inter.interpolate_linear(&other.next, f)
        };

        if (self.timestamp - other.timestamp).abs() > 0.00001 {
            panic!(
                "Timestamps of interpolated frames don't match! {:?} vs {:?}",
                self.timestamp, other.timestamp
            );
        }

        let out = Self {
            timestamp: self.timestamp,
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
        };

        out
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
            if let Some(other_bone_id) = other.paths.get(path) {
                result.add_bone(
                    self.bones[*bone_id].interpolate_linear(&other.bones[*other_bone_id], f),
                    path.clone(),
                );
            } else {
                result.add_bone(self.bones[*bone_id].clone(), path.clone());
            }
        }

        for (path, bone_id) in other.paths.iter() {
            if self.paths.contains_key(path) {
                continue;
            }
            result.add_bone(other.bones[*bone_id].clone(), path.clone());
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_value_frame_nest_1() {
        let frame_1 = ValueFrame {
            timestamp: 0.5,
            prev: Vec3::new(0., 0., 0.),
            prev_timestamp: 0.,
            next: Vec3::new(1., 1., 1.),
            next_timestamp: 1.,
            next_is_wrapped: false,
        };
        let frame_2 = ValueFrame {
            timestamp: 0.5,
            prev: Vec3::new(0., 0., 0.),
            prev_timestamp: 0.2,
            next: Vec3::new(1., 1., 1.),
            next_timestamp: 0.8,
            next_is_wrapped: false,
        };

        let interpolated_0 = frame_1.interpolate_linear(&frame_2, 0.);
        let interpolated_half = frame_1.interpolate_linear(&frame_2, 0.5);
        let interpolated_1 = frame_1.interpolate_linear(&frame_2, 1.);

        let expected_0 = ValueFrame {
            timestamp: 0.5,
            prev: Vec3::new(0.2, 0.2, 0.2),
            prev_timestamp: 0.2,
            next: Vec3::new(0.8, 0.8, 0.8),
            next_timestamp: 0.8,
            next_is_wrapped: false,
        };

        let expected_half = ValueFrame {
            timestamp: 0.5,
            prev: Vec3::new(0.1, 0.1, 0.1),
            prev_timestamp: 0.2,
            next: Vec3::new(0.9, 0.9, 0.9),
            next_timestamp: 0.8,
            next_is_wrapped: false,
        };

        let expected_1 = ValueFrame {
            timestamp: 0.5,
            prev: Vec3::new(0.0, 0.0, 0.0),
            prev_timestamp: 0.2,
            next: Vec3::new(1.0, 1.0, 1.0),
            next_timestamp: 0.8,
            next_is_wrapped: false,
        };

        assert_eq!(expected_0, interpolated_0);
        assert_eq!(expected_1, interpolated_1);
        assert_eq!(expected_half, interpolated_half);
    }

    #[test]
    fn test_interpolate_value_frame_nest_2() {
        let frame_2 = ValueFrame {
            timestamp: 0.5,
            prev: Vec3::new(0., 0., 0.),
            prev_timestamp: 0.,
            next: Vec3::new(1., 1., 1.),
            next_timestamp: 1.,
            next_is_wrapped: false,
        };
        let frame_1 = ValueFrame {
            timestamp: 0.5,
            prev: Vec3::new(0., 0., 0.),
            prev_timestamp: 0.2,
            next: Vec3::new(1., 1., 1.),
            next_timestamp: 0.8,
            next_is_wrapped: false,
        };

        let interpolated_1 = frame_1.interpolate_linear(&frame_2, 0.);
        let interpolated_half = frame_1.interpolate_linear(&frame_2, 0.5);
        let interpolated_0 = frame_1.interpolate_linear(&frame_2, 1.);

        let expected_0 = ValueFrame {
            timestamp: 0.5,
            prev: Vec3::new(0.2, 0.2, 0.2),
            prev_timestamp: 0.2,
            next: Vec3::new(0.8, 0.8, 0.8),
            next_timestamp: 0.8,
            next_is_wrapped: false,
        };

        let expected_half = ValueFrame {
            timestamp: 0.5,
            prev: Vec3::new(0.1, 0.1, 0.1),
            prev_timestamp: 0.2,
            next: Vec3::new(0.9, 0.9, 0.9),
            next_timestamp: 0.8,
            next_is_wrapped: false,
        };

        let expected_1 = ValueFrame {
            timestamp: 0.5,
            prev: Vec3::new(0.0, 0.0, 0.0),
            prev_timestamp: 0.2,
            next: Vec3::new(1.0, 1.0, 1.0),
            next_timestamp: 0.8,
            next_is_wrapped: false,
        };

        assert_eq!(expected_0, interpolated_0);
        assert_eq!(expected_1, interpolated_1);
        assert_eq!(expected_half, interpolated_half);
    }
}
