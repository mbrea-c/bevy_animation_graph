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
pub struct IntoEulerNode {
    pub mode: EulerRot,
}

impl IntoEulerNode {
    pub const INPUT: &'static str = "quat";
    pub const OUTPUT: &'static str = "euler";

    pub fn new(mode: EulerRot) -> Self {
        Self { mode }
    }
}

impl NodeLike for IntoEulerNode {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let quat: Quat = ctx.data_back(Self::INPUT)?.as_quat()?;

        let (x, y, z) = quat.to_euler(self.mode);
        let output = Vec3::new(x, y, z);

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT, DataSpec::Quat);
        ctx.add_output_data(Self::OUTPUT, DataSpec::Vec3);
        Ok(())
    }

    fn display_name(&self) -> String {
        "Quat into Euler".into()
    }
}
