use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct DummyNode;

impl DummyNode {
    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for DummyNode {
    fn clone_value(&self) -> Box<dyn NodeLike> {
        Box::new(self.clone())
    }

    fn display_name(&self) -> String {
        "Dummy".into()
    }
}
