use bevy::reflect::Reflect;
use bevy::reflect::prelude::ReflectDefault;

use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::{DataSpec, DataValue};
use crate::core::errors::GraphError;
use crate::core::ragdoll::configuration::RagdollConfig;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct ConstRagdollConfig {
    pub value: RagdollConfig,
}

impl ConstRagdollConfig {
    pub const OUTPUT: &'static str = "out";

    pub fn new(value: RagdollConfig) -> Self {
        Self { value }
    }
}

impl NodeLike for ConstRagdollConfig {
    fn display_name(&self) -> String {
        "Ragdoll Config".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        ctx.set_data_fwd(Self::OUTPUT, DataValue::RagdollConfig(self.value.clone()));
        Ok(())
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::RagdollConfig)].into()
    }
}
