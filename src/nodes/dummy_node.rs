use crate::core::animation_node::{AnimationNode, AnimationNodeType, CustomNode, NodeLike};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
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
    fn display_name(&self) -> String {
        "Dummy".into()
    }
}
