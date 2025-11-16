use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::SpecContext;
use crate::prelude::new_context::NodeContext;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
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
