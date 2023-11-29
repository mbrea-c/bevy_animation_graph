use crate::core::animation_graph::{
    EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, CustomNode, NodeLike};
use crate::core::caches::{DurationCache, EdgePathCache, ParameterCache, TimeCache};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
pub struct DummyNode {}

impl DummyNode {
    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: String) -> AnimationNode {
        AnimationNode::new_from_nodetype(name, AnimationNodeType::Custom(CustomNode::new(self)))
    }
}

impl NodeLike for DummyNode {
    fn parameter_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<NodeInput, Option<f32>>,
        parameters: &ParameterCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> Option<f32> {
        None
    }

    fn time_pass(
        &self,
        input: TimeState,
        parameters: &ParameterCache,
        durations: &DurationCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeInput, TimeUpdate> {
        HashMap::new()
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        parameters: &ParameterCache,
        durations: &DurationCache,
        time: &TimeCache,
        _last_cache: Option<&EdgePathCache>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        HashMap::new()
    }

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::new()
    }

    fn duration_input_spec(&self) -> HashMap<NodeInput, ()> {
        HashMap::new()
    }

    fn time_dependent_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn time_dependent_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::new()
    }
}
