use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_graph::PinId,
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataValue,
    errors::GraphError,
    utils::sorted_map::SortedMap,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct Constants {
    pub constants: SortedMap<PinId, DataValue>,
}

impl NodeLike for Constants {
    fn display_name(&self) -> String {
        "Constants".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        for (k, v) in self.constants.iter() {
            ctx.set_data_fwd(k, v.clone());
        }

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        for (k, v) in self.constants.iter() {
            ctx.add_output_data(k, v.into());
        }

        Ok(())
    }
}
