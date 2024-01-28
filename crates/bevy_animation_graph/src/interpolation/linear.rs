use crate::core::frame::{
    BoneFrame, InnerPoseFrame, PoseFrame, PoseFrameData, PoseSpec, ValueFrame,
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

impl InterpolateLinear for Transform {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        Transform {
            translation: self.translation.interpolate_linear(&other.translation, f),
            rotation: self.rotation.interpolate_linear(&other.rotation, f),
            scale: self.scale.interpolate_linear(&other.scale, f),
        }
    }
}

impl<T: InterpolateLinear + FromReflect + TypePath + std::fmt::Debug + Clone> InterpolateLinear
    for ValueFrame<T>
{
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        self.merge_linear(other, |l, r| l.interpolate_linear(r, f))
    }
}

impl InterpolateLinear for BoneFrame {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        let mut result = Self::default();

        // TODO: Maybe we should blend with rest pose whenever one channel is missing?

        match (&self.rotation, &other.rotation) {
            (Some(a), Some(b)) => {
                result.rotation = Some(a.interpolate_linear(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.rotation = Some(b.clone()),
            (Some(a), None) => result.rotation = Some(a.clone()),
        }

        match (&self.translation, &other.translation) {
            (Some(a), Some(b)) => {
                result.translation = Some(a.interpolate_linear(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.translation = Some(b.clone()),
            (Some(a), None) => result.translation = Some(a.clone()),
        }

        match (&self.scale, &other.scale) {
            (Some(a), Some(b)) => {
                result.scale = Some(a.interpolate_linear(b, f));
            }
            (None, None) => {}
            (None, Some(b)) => result.scale = Some(b.clone()),
            (Some(a), None) => result.scale = Some(a.clone()),
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

impl InterpolateLinear for InnerPoseFrame {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        let mut result = InnerPoseFrame::default();

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

impl InterpolateLinear for PoseFrameData {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        match (self, other) {
            (PoseFrameData::BoneSpace(f1), PoseFrameData::BoneSpace(f2)) => {
                PoseFrameData::BoneSpace(
                    f1.inner_ref().interpolate_linear(f2.inner_ref(), f).into(),
                )
            }
            (PoseFrameData::CharacterSpace(f1), PoseFrameData::CharacterSpace(f2)) => {
                PoseFrameData::CharacterSpace(
                    f1.inner_ref().interpolate_linear(f2.inner_ref(), f).into(),
                )
            }
            (PoseFrameData::GlobalSpace(f1), PoseFrameData::GlobalSpace(f2)) => {
                PoseFrameData::GlobalSpace(
                    f1.inner_ref().interpolate_linear(f2.inner_ref(), f).into(),
                )
            }
            _ => {
                panic!(
                    "Tried to chain {:?} with {:?}",
                    PoseSpec::from(self),
                    PoseSpec::from(other)
                )
            }
        }
    }
}

impl InterpolateLinear for PoseFrame {
    fn interpolate_linear(&self, other: &Self, f: f32) -> Self {
        Self {
            data: self.data.interpolate_linear(&other.data, f),
            timestamp: self.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_value_frame_nest_1() {
        let frame_1 = ValueFrame {
            prev: Vec3::new(0., 0., 0.),
            prev_timestamp: 0.,
            next: Vec3::new(1., 1., 1.),
            next_timestamp: 1.,
            next_is_wrapped: false,
            prev_is_wrapped: false,
        };
        let frame_2 = ValueFrame {
            prev: Vec3::new(0., 0., 0.),
            prev_timestamp: 0.2,
            next: Vec3::new(1., 1., 1.),
            next_timestamp: 0.8,
            next_is_wrapped: false,
            prev_is_wrapped: false,
        };

        let interpolated_0 = frame_1.interpolate_linear(&frame_2, 0.);
        let interpolated_half = frame_1.interpolate_linear(&frame_2, 0.5);
        let interpolated_1 = frame_1.interpolate_linear(&frame_2, 1.);

        let expected_0 = ValueFrame {
            prev: Vec3::new(0.2, 0.2, 0.2),
            prev_timestamp: 0.2,
            next: Vec3::new(0.8, 0.8, 0.8),
            next_timestamp: 0.8,
            next_is_wrapped: false,
            prev_is_wrapped: false,
        };

        let expected_half = ValueFrame {
            prev: Vec3::new(0.1, 0.1, 0.1),
            prev_timestamp: 0.2,
            next: Vec3::new(0.9, 0.9, 0.9),
            next_timestamp: 0.8,
            next_is_wrapped: false,
            prev_is_wrapped: false,
        };

        let expected_1 = ValueFrame {
            prev: Vec3::new(0.0, 0.0, 0.0),
            prev_timestamp: 0.2,
            next: Vec3::new(1.0, 1.0, 1.0),
            next_timestamp: 0.8,
            next_is_wrapped: false,
            prev_is_wrapped: false,
        };

        assert_eq!(expected_0, interpolated_0);
        assert_eq!(expected_1, interpolated_1);
        assert_eq!(expected_half, interpolated_half);
    }

    #[test]
    fn test_interpolate_value_frame_nest_2() {
        let frame_2 = ValueFrame {
            prev: Vec3::new(0., 0., 0.),
            prev_timestamp: 0.,
            next: Vec3::new(1., 1., 1.),
            next_timestamp: 1.,
            next_is_wrapped: false,
            prev_is_wrapped: false,
        };
        let frame_1 = ValueFrame {
            prev: Vec3::new(0., 0., 0.),
            prev_timestamp: 0.2,
            next: Vec3::new(1., 1., 1.),
            next_timestamp: 0.8,
            next_is_wrapped: false,
            prev_is_wrapped: false,
        };

        let interpolated_1 = frame_1.interpolate_linear(&frame_2, 0.);
        let interpolated_half = frame_1.interpolate_linear(&frame_2, 0.5);
        let interpolated_0 = frame_1.interpolate_linear(&frame_2, 1.);

        let expected_0 = ValueFrame {
            prev: Vec3::new(0.2, 0.2, 0.2),
            prev_timestamp: 0.2,
            next: Vec3::new(0.8, 0.8, 0.8),
            next_timestamp: 0.8,
            next_is_wrapped: false,
            prev_is_wrapped: false,
        };

        let expected_half = ValueFrame {
            prev: Vec3::new(0.1, 0.1, 0.1),
            prev_timestamp: 0.2,
            next: Vec3::new(0.9, 0.9, 0.9),
            next_timestamp: 0.8,
            next_is_wrapped: false,
            prev_is_wrapped: false,
        };

        let expected_1 = ValueFrame {
            prev: Vec3::new(0.0, 0.0, 0.0),
            prev_timestamp: 0.2,
            next: Vec3::new(1.0, 1.0, 1.0),
            next_timestamp: 0.8,
            next_is_wrapped: false,
            prev_is_wrapped: false,
        };

        assert_eq!(expected_0, interpolated_0);
        assert_eq!(expected_1, interpolated_1);
        assert_eq!(expected_half, interpolated_half);
    }
}
