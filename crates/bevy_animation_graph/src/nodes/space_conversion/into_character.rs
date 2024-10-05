use crate::{
    core::{
        animation_graph::{PinMap, TimeUpdate},
        animation_node::{AnimationNode, AnimationNodeType, NodeLike},
        duration_data::DurationData,
        errors::GraphError,
        pose::{Pose, PoseSpec},
    },
    prelude::{PassContext, SpecContext},
};
use bevy::reflect::{std_traits::ReflectDefault, Reflect};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
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
    fn duration(&self, mut ctx: PassContext) -> Result<Option<DurationData>, GraphError> {
        Ok(Some(ctx.duration_back(Self::POSE_IN)?))
    }

    fn pose_pass(
        &self,
        _time_update: TimeUpdate,
        mut _ctx: PassContext,
    ) -> Result<Option<Pose>, GraphError> {
        // let in_pose = ctx.pose_back(Self::POSE_IN, time_update)?;
        // Ok(Some(PoseFrame {
        //     timestamp: in_pose.timestamp,
        //     data: PoseFrameData::CharacterSpace(match &in_pose.data {
        //         PoseFrameData::BoneSpace(data) => ctx.bone_to_character(data),
        //         PoseFrameData::CharacterSpace(data) => data.clone(),
        //         PoseFrameData::GlobalSpace(data) => ctx.global_to_character(data),
        //     }),
        // }))
        todo!()
    }

    fn pose_input_spec(&self, _ctx: SpecContext) -> PinMap<PoseSpec> {
        [(Self::POSE_IN.into(), PoseSpec::Any)].into()
    }

    fn pose_output_spec(&self, _ctx: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::CharacterSpace)
    }

    fn display_name(&self) -> String {
        "* → Character".into()
    }
}
