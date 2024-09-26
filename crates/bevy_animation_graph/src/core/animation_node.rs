use super::{
    animation_graph::{PinId, PinMap},
    edge_data::DataSpec,
    errors::GraphError,
};
use crate::{
    nodes::DummyNode,
    prelude::{PassContext, SpecContext},
};
use bevy::{reflect::prelude::*, utils::HashMap};
use core::fmt;

#[reflect_trait]
pub trait NodeLike: Send + Sync + Reflect {
    fn duration(&self, _ctx: PassContext) -> Result<(), GraphError> {
        Ok(())
    }

    fn update(&self, _ctx: PassContext) -> Result<(), GraphError> {
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        PinMap::new()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        PinMap::new()
    }

    fn time_input_spec(&self, _ctx: SpecContext) -> PinMap<()> {
        PinMap::new()
    }

    /// Specify whether or not a node outputs a pose, and which space the pose is in
    fn time_output_spec(&self, _ctx: SpecContext) -> Option<()> {
        None
    }

    /// The name of this node.
    fn display_name(&self) -> String;

    /// The order of the input pins. This way, you can mix time and data pins in the UI.
    fn input_pin_ordering(&self) -> PinOrdering {
        PinOrdering::default()
    }

    /// The order of the output pins. This way, you can mix time and data pins in the UI.
    fn output_pin_ordering(&self) -> PinOrdering {
        PinOrdering::default()
    }
}

#[derive(Clone, Reflect, Debug, Default)]
pub struct PinOrdering {
    keys: HashMap<PinId, usize>,
}

impl PinOrdering {
    pub fn new(keys: impl Into<HashMap<PinId, usize>>) -> Self {
        Self { keys: keys.into() }
    }

    pub fn pin_key(&self, pin_id: &PinId) -> usize {
        self.keys.get(pin_id).copied().unwrap_or(0)
    }
}

#[derive(Reflect)]
pub struct AnimationNode {
    pub name: String,
    pub node: Box<dyn NodeLike>,
    #[reflect(ignore)]
    pub should_debug: bool,
}

impl AnimationNode {
    #[must_use]
    pub fn new(name: impl Into<String>, node: Box<dyn NodeLike>) -> Self {
        Self {
            name: name.into(),
            node: node.into(),
            should_debug: false,
        }
    }
}

impl Default for AnimationNode {
    fn default() -> Self {
        Self::new("", Box::new(DummyNode))
    }
}

impl Clone for AnimationNode {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            node: self.node.type_id(), // todo
            should_debug: self.should_debug,
        }
    }
}

impl fmt::Debug for AnimationNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AnimationNode")
            .field("name", &self.name)
            .field("node_type", &self.node.reflect_short_type_path())
            .finish()
    }
}
