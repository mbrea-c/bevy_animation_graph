use crate::core::animation_graph::{
    OptParamSpec, ParamSpec, ParamValue, PinId, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, CustomNode, NodeLike};
use crate::core::frame::PoseFrame;
use crate::prelude::{DurationData, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Reflect, Clone, Debug, Default)]
pub struct DummyNode {}

impl DummyNode {
    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(
            name.into(),
            AnimationNodeType::Custom(CustomNode::new(self)),
        )
    }
}

impl NodeLike for DummyNode {
    fn parameter_pass(
        &self,
        _inputs: HashMap<PinId, ParamValue>,
        _: PassContext,
    ) -> HashMap<PinId, ParamValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        _inputs: HashMap<PinId, Option<f32>>,
        _: PassContext,
    ) -> Option<DurationData> {
        None
    }

    fn time_pass(&self, _input: TimeState, _: PassContext) -> HashMap<PinId, TimeUpdate> {
        HashMap::new()
    }

    fn time_dependent_pass(
        &self,
        _inputs: HashMap<PinId, PoseFrame>,
        _: PassContext,
    ) -> Option<PoseFrame> {
        None
    }

    fn parameter_input_spec(&self, _: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self, _: SpecContext) -> HashMap<PinId, ParamSpec> {
        HashMap::new()
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashSet<PinId> {
        HashSet::new()
    }

    fn pose_output_spec(&self, _: SpecContext) -> bool {
        false
    }

    fn display_name(&self) -> String {
        "Dummy".into()
    }
}
