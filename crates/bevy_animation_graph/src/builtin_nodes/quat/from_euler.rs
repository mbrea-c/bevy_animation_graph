use bevy::prelude::*;

use crate::core::{
    animation_graph::PinMap,
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

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::INPUT.into(), DataSpec::Vec3)].into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Quat)].into()
    }

    fn display_name(&self) -> String {
        "Quat from Euler".into()
    }
}
