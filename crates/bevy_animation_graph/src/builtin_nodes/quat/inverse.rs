use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::DataSpec;
use crate::core::errors::GraphError;
use bevy::prelude::*;

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
