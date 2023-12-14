use crate::core::animation_graph::{
    OptParamSpec, ParamSpec, ParamValue, PinId, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::frame::PoseFrame;
use crate::flipping::FlipXBySuffix;
use crate::prelude::{DurationData, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Reflect, Clone, Debug)]
pub struct FlipLRNode {}

impl Default for FlipLRNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FlipLRNode {
    pub const INPUT: &'static str = "Pose In";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::FlipLR(self))
    }
}

impl NodeLike for FlipLRNode {
    fn parameter_pass(
        &self,
        _inputs: HashMap<PinId, ParamValue>,
        _: PassContext,
    ) -> HashMap<PinId, ParamValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<PinId, Option<f32>>,
        _: PassContext,
    ) -> Option<DurationData> {
        Some(*inputs.get(Self::INPUT).unwrap())
    }

    fn time_pass(&self, input: TimeState, _: PassContext) -> HashMap<PinId, TimeUpdate> {
        // Propagate the time update without modification
        HashMap::from([(Self::INPUT.into(), input.update)])
    }

    fn time_dependent_pass(
        &self,
        mut inputs: HashMap<PinId, PoseFrame>,
        _: PassContext,
    ) -> Option<PoseFrame> {
        let in_pose_frame = inputs.remove(Self::INPUT).unwrap();
        let flipped_pose_frame = in_pose_frame.flipped_by_suffix(" R".into(), " L".into());

        Some(flipped_pose_frame)
    }

    fn parameter_input_spec(&self, _: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self, _: SpecContext) -> HashMap<PinId, ParamSpec> {
        HashMap::new()
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashSet<PinId> {
        HashSet::from([Self::INPUT.into()])
    }

    fn pose_output_spec(&self, _: SpecContext) -> bool {
        true
    }

    fn display_name(&self) -> String {
        "🯈|🯇 Flip Left/Right".into()
    }
}
