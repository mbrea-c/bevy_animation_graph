use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_graph::PinMap,
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct InvertQuatNode;

impl InvertQuatNode {
    pub const INPUT: &'static str = "quat";
    pub const OUTPUT: &'static str = "inverse";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for InvertQuatNode {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input: Quat = ctx.data_back(Self::INPUT)?.as_quat()?;
        let output: Quat = input.inverse();

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::INPUT.into(), DataSpec::Quat)].into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Quat)].into()
    }

    fn display_name(&self) -> String {
        "Invert Quat".into()
    }
}
