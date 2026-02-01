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
pub struct FromEulerNode {
    pub mode: EulerRot,
}

impl FromEulerNode {
    pub const INPUT: &'static str = "euler";
    pub const OUTPUT: &'static str = "quat";

    pub fn new(mode: EulerRot) -> Self {
        Self { mode }
    }
}

impl NodeLike for FromEulerNode {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let Vec3 { x, y, z } = ctx.data_back(Self::INPUT)?.as_vec3()?;

        let output = Quat::from_euler(self.mode, x, y, z);

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT, DataSpec::Vec3);
        ctx.add_output_data(Self::OUTPUT, DataSpec::Quat);
        Ok(())
    }

    fn display_name(&self) -> String {
        "Quat from Euler".into()
    }
}
