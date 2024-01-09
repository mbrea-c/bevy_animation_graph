use crate::{
    core::{
        animation_graph::{PinId, TimeUpdate},
        animation_node::{AnimationNode, AnimationNodeType, NodeLike},
        duration_data::DurationData,
        frame::{PoseFrame, PoseFrameData, PoseSpec},
        space_conversion::{bone_to_character, global_to_character},
    },
    prelude::{PassContext, SpecContext},
};
use bevy::{reflect::Reflect, utils::HashMap};

#[derive(Reflect, Clone, Debug, Default)]
pub struct IntoCharacterSpaceNode {}

impl IntoCharacterSpaceNode {
    pub const POSE_IN: &'static str = "Pose In";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::IntoCharacterSpace(self))
    }
}

impl NodeLike for IntoCharacterSpaceNode {
    fn duration_pass(&self, mut ctx: PassContext) -> Option<DurationData> {
        Some(ctx.duration_back(Self::POSE_IN))
    }

    fn pose_pass(&self, time_update: TimeUpdate, mut ctx: PassContext) -> Option<PoseFrame> {
        let in_pose = ctx.pose_back(Self::POSE_IN, time_update);
        Some(PoseFrame {
            timestamp: in_pose.timestamp,
            data: PoseFrameData::CharacterSpace(match &in_pose.data {
                PoseFrameData::BoneSpace(data) => bone_to_character(data, ctx),
                PoseFrameData::CharacterSpace(data) => data.clone(),
                PoseFrameData::GlobalSpace(data) => global_to_character(data, ctx),
            }),
        })
    }

    fn pose_input_spec(&self, _ctx: SpecContext) -> HashMap<PinId, PoseSpec> {
        HashMap::from([(Self::POSE_IN.into(), PoseSpec::Any)])
    }

    fn pose_output_spec(&self, _ctx: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::CharacterSpace)
    }

    fn display_name(&self) -> String {
        "* → Character".into()
    }
}
