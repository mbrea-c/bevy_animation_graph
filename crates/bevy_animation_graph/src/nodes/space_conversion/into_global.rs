use crate::{
    core::{
        animation_graph::{PinMap, TimeUpdate},
        animation_node::{AnimationNode, AnimationNodeType, NodeLike},
        duration_data::DurationData,
        frame::{PoseFrame, PoseFrameData, PoseSpec},
        space_conversion::SpaceConversion,
    },
    prelude::{PassContext, SpecContext},
};
use bevy::reflect::{std_traits::ReflectDefault, Reflect};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct IntoGlobalSpaceNode {}

impl IntoGlobalSpaceNode {
    pub const POSE_IN: &'static str = "Pose In";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::IntoGlobalSpace(self))
    }
}

impl NodeLike for IntoGlobalSpaceNode {
    fn duration_pass(&self, mut ctx: PassContext) -> Option<DurationData> {
        Some(ctx.duration_back(Self::POSE_IN))
    }

    fn pose_pass(&self, time_update: TimeUpdate, mut ctx: PassContext) -> Option<PoseFrame> {
        let in_pose = ctx.pose_back(Self::POSE_IN, time_update);
        Some(PoseFrame {
            timestamp: in_pose.timestamp,
            data: PoseFrameData::GlobalSpace(match &in_pose.data {
                PoseFrameData::BoneSpace(data) => ctx.bone_to_global(data),
                PoseFrameData::CharacterSpace(data) => ctx.character_to_global(data),
                PoseFrameData::GlobalSpace(data) => data.clone(),
            }),
        })
    }

    fn pose_input_spec(&self, _ctx: SpecContext) -> PinMap<PoseSpec> {
        [(Self::POSE_IN.into(), PoseSpec::Any)].into()
    }

    fn pose_output_spec(&self, _ctx: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::GlobalSpace)
    }

    fn display_name(&self) -> String {
        "* → Global".into()
    }
}
