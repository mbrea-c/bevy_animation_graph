use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
};

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct SlerpQuatNode;

impl SlerpQuatNode {
    pub const INPUT_A: &'static str = "a";
    pub const INPUT_B: &'static str = "b";
    pub const INPUT_FACTOR: &'static str = "factor";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for SlerpQuatNode {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let a: Quat = ctx.data_back(Self::INPUT_A)?.as_quat()?;
        let b: Quat = ctx.data_back(Self::INPUT_B)?.as_quat()?;
        let factor: f32 = ctx.data_back(Self::INPUT_FACTOR)?.as_f32()?;

        let output = Quat::slerp(a, b, factor);

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx //
            .add_input_data(Self::INPUT_A, DataSpec::Quat)
            .add_input_data(Self::INPUT_B, DataSpec::Quat)
            .add_input_data(Self::INPUT_FACTOR, DataSpec::F32);
        ctx.add_output_data(Self::OUTPUT, DataSpec::Quat);
        Ok(())
    }

    fn display_name(&self) -> String {
        "Slerp Quat".into()
    }
}
