use crate::{
    core::{
        animation_graph::{PinMap, TimeUpdate},
        animation_node::{AnimationNode, AnimationNodeType, NodeLike},
        duration_data::DurationData,
        errors::GraphError,
        frame::{PoseFrame, PoseFrameData, PoseSpec},
        space_conversion::SpaceConversion,
    },
    prelude::{PassContext, SpecContext},
};
use bevy::reflect::{std_traits::ReflectDefault, Reflect};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct IntoBoneSpaceNode {}

impl IntoBoneSpaceNode {
    pub const POSE_IN: &'static str = "Pose In";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::IntoBoneSpace(self))
    }
}

impl NodeLike for IntoBoneSpaceNode {
    fn duration_pass(&self, mut ctx: PassContext) -> Result<Option<DurationData>, GraphError> {
        Ok(Some(ctx.duration_back(Self::POSE_IN)?))
    }

    fn pose_pass(
        &self,
        time_update: TimeUpdate,
        mut ctx: PassContext,
    ) -> Result<Option<PoseFrame>, GraphError> {
        let in_pose = ctx.pose_back(Self::POSE_IN, time_update)?;
        Ok(Some(PoseFrame {
            timestamp: in_pose.timestamp,
            data: PoseFrameData::BoneSpace(match &in_pose.data {
                PoseFrameData::BoneSpace(data) => data.clone(),
                PoseFrameData::CharacterSpace(data) => ctx.character_to_bone(data),
                PoseFrameData::GlobalSpace(data) => ctx.global_to_bone(data),
            }),
        }))
    }

    fn pose_input_spec(&self, _ctx: SpecContext) -> PinMap<PoseSpec> {
        [(Self::POSE_IN.into(), PoseSpec::Any)].into()
    }

    fn pose_output_spec(&self, _ctx: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::BoneSpace)
    }

    fn display_name(&self) -> String {
        "* → Bone".into()
    }
}
