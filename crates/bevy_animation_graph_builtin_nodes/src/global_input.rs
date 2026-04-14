use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_graph::PinId,
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpecWithOptionalDefault,
    errors::GraphError,
    utils::sorted_map::SortedMap,
};

/// Exposes global inputs anywhere
///
/// Global inputs are specified on an animation graph player by gameplay logic and can be accessed
/// by any animation graph during evaluation, without having to be "wired" through nested graphs
/// and FSMs.
#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct GlobalInput {
    /// Which global inputs to expose in this node, along with an optional default
    pub values: SortedMap<PinId, DataSpecWithOptionalDefault>,
}

impl NodeLike for GlobalInput {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        for (key, value) in self.values.iter() {
            let val = ctx
                .graph_context
                .global_input_data
                .get(key)
                .or(value.default.as_ref())
                .cloned()
                .ok_or_else(|| GraphError::GlobalInputDataMissing(key.clone()))?;

            ctx.set_data_fwd(key, val);
        }

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        for (pin, val) in self.values.iter() {
            ctx.add_output_data(pin, val.spec);
        }

        Ok(())
    }

    fn display_name(&self) -> String {
        "Global input".into()
    }
}
