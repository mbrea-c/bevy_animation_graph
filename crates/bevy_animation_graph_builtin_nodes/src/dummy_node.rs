use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct DummyNode;

impl DummyNode {
    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for DummyNode {
    fn display_name(&self) -> String {
        "Dummy".into()
    }

    fn update(&self, _: NodeContext) -> Result<(), GraphError> {
        Ok(())
    }

    fn spec(&self, _: SpecContext) -> Result<(), GraphError> {
        Ok(())
    }
}
