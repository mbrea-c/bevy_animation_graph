use bevy::prelude::*;

use crate::core::animation_node::{NodeLike, ReflectNodeLike};

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
}
