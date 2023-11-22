use crate::animation::{
    AnimationNode, CustomNode, EdgeSpec, EdgeValue, NodeInput, NodeLike, NodeOutput,
};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
pub struct DummyNode {}

impl DummyNode {
    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self) -> AnimationNode {
        AnimationNode::Custom(CustomNode::new(self))
    }
}

impl NodeLike for DummyNode {
    fn duration(&mut self, _input_durations: HashMap<NodeInput, Option<f32>>) -> Option<f32> {
        None
    }

    fn forward(&self, _time: f32) -> HashMap<NodeInput, f32> {
        HashMap::from([])
    }

    fn backward(
        &self,
        _time: f32,
        _inputs: HashMap<NodeInput, EdgeValue>,
    ) -> HashMap<NodeOutput, EdgeValue> {
        HashMap::from([])
    }

    fn input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([])
    }

    fn output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([])
    }
}
