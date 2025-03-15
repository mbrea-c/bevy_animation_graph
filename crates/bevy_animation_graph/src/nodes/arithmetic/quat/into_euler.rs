use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
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
    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let quat: Quat = ctx.data_back(Self::INPUT)?.as_quat()?;

        let (x, y, z) = quat.to_euler(self.mode);
        let output = Vec3::new(x, y, z);

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::INPUT.into(), DataSpec::Quat)].into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Vec3)].into()
    }

    fn display_name(&self) -> String {
        "Quat into Euler".into()
    }
}
