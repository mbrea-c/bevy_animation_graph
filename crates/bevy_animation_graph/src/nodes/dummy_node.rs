use crate::core::animation_node::NodeLike;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
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
