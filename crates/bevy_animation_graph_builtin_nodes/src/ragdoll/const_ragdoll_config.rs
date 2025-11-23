use bevy::reflect::{Reflect, prelude::ReflectDefault};
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::{DataSpec, DataValue},
    errors::GraphError,
    ragdoll::configuration::RagdollConfig,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
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

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_output_data(Self::OUTPUT, DataSpec::RagdollConfig);
        Ok(())
    }
}
