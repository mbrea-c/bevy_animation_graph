use crate::core::animation_clip::EntityPath;
use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::errors::GraphError;
use crate::core::frame::{
    BoneFrame, BonePoseFrame, PoseFrame, PoseFrameData, PoseSpec, ValueFrame,
};
use crate::core::space_conversion::SpaceConversion;
use crate::prelude::{OptParamSpec, ParamSpec, PassContext, SampleLinearAt, SpecContext};
use crate::utils::unwrap::Unwrap;
use bevy::math::Quat;
use bevy::reflect::std_traits::ReflectDefault;
use bevy::reflect::Reflect;
use bevy::transform::components::Transform;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum RotationMode {
    #[default]
    Blend,
    Compose,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum RotationSpace {
    #[default]
    Local,
    Character,
    Global,
}

#[derive(Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum ChainDecay {
    #[default]
    Linear,
}

#[derive(Reflect, Clone, Debug)]
#[reflect(Default)]
pub struct RotationNode {
    pub application_mode: RotationMode,
    pub rotation_space: RotationSpace,
    pub chain_decay: ChainDecay,
    pub chain_length: usize,
    pub base_weight: f32,
}

impl Default for RotationNode {
    fn default() -> Self {
        Self {
            application_mode: RotationMode::Blend,
            rotation_space: RotationSpace::Local,
            chain_decay: ChainDecay::Linear,
            chain_length: 1,
            base_weight: 1.0,
        }
    }
}

impl RotationNode {
    pub const INPUT: &'static str = "Pose In";
    pub const TARGET: &'static str = "Bone Mask";
    pub const ROTATION: &'static str = "Rotation";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new(
        mode: RotationMode,
        space: RotationSpace,
        chain_decay: ChainDecay,
        chain_length: usize,
        base_weight: f32,
    ) -> Self {
        Self {
            application_mode: mode,
            rotation_space: space,
            chain_decay,
            chain_length,
            base_weight,
        }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Rotation(self))
    }
}

impl NodeLike for RotationNode {
    fn duration_pass(&self, mut ctx: PassContext) -> Result<Option<DurationData>, GraphError> {
        Ok(Some(ctx.duration_back(Self::INPUT)?))
    }

    fn pose_pass(
        &self,
        input: TimeUpdate,
        mut ctx: PassContext,
    ) -> Result<Option<PoseFrame>, GraphError> {
        let mut target: EntityPath = ctx.parameter_back(Self::TARGET)?.unwrap();
        let rotation: Quat = ctx.parameter_back(Self::ROTATION)?.unwrap();
        let pose = ctx.pose_back(Self::INPUT, input)?;
        let time = pose.timestamp;
        let mut pose: BonePoseFrame = pose.data.unwrap();
        let inner_pose = pose.inner_mut();

        if !inner_pose.paths.contains_key(&target) {
            inner_pose.add_bone(BoneFrame::default(), target.clone());
        }

        // build bone chain
        let mut chain = vec![target.clone()];
        while let Some(parent) = target.parent() {
            if chain.len() >= self.chain_length {
                break;
            }

            chain.insert(0, parent.clone());
            target = parent;
        }

        for (i, target) in chain.into_iter().enumerate() {
            let percent = (i + 1) as f32 / self.chain_length.max(1) as f32 * self.base_weight;
            let rotation_bone_space = match self.rotation_space {
                RotationSpace::Local => rotation,
                RotationSpace::Character => {
                    if let Some(parent) = target.parent() {
                        ctx.root_to_bone_space(
                            Transform::from_rotation(rotation),
                            inner_pose,
                            parent,
                            time,
                        )
                        .rotation
                    } else {
                        rotation
                    }
                }
                RotationSpace::Global => {
                    if let Some(parent) = target.parent() {
                        ctx.global_to_bone_space(
                            Transform::from_rotation(rotation),
                            inner_pose,
                            parent,
                            time,
                        )
                        .rotation
                    } else {
                        ctx.transform_global_to_character(Transform::from_rotation(rotation))
                            .rotation
                    }
                }
            };

            let mut bone_frame = inner_pose
                .paths
                .get(&target)
                .and_then(|bone_id| inner_pose.bones.get_mut(*bone_id).cloned())
                .unwrap_or_else(|| BoneFrame::default());

            if let Some(mut rot_frame) = bone_frame.rotation {
                let rot = rot_frame.sample_linear_at(time);
                let rotation = match self.application_mode {
                    RotationMode::Blend => rot.slerp(rotation_bone_space, percent),
                    RotationMode::Compose => {
                        Quat::IDENTITY.slerp(rotation_bone_space, percent) * rot
                    }
                };
                rot_frame.prev = rotation;
                rot_frame.next = rotation;
                bone_frame.rotation = Some(rot_frame);
            } else {
                bone_frame.rotation = Some(ValueFrame {
                    prev: rotation_bone_space,
                    prev_timestamp: time - 0.1,
                    next: rotation_bone_space,
                    next_timestamp: time + 0.1,
                    prev_is_wrapped: false,
                    next_is_wrapped: false,
                });
            }

            inner_pose.add_bone(bone_frame, target);
        }

        Ok(Some(PoseFrame {
            data: PoseFrameData::BoneSpace(pose),
            timestamp: time,
        }))
    }

    fn parameter_input_spec(&self, _ctx: SpecContext) -> PinMap<OptParamSpec> {
        [
            (Self::TARGET.into(), ParamSpec::EntityPath.into()),
            (Self::ROTATION.into(), ParamSpec::Quat.into()),
        ]
        .into()
    }

    fn pose_input_spec(&self, _: SpecContext) -> PinMap<PoseSpec> {
        [(Self::INPUT.into(), PoseSpec::BoneSpace)].into()
    }

    fn pose_output_spec(&self, _: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::BoneSpace)
    }

    fn display_name(&self) -> String {
        "⮩ Rotation".into()
    }
}
