use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct DecomposeVec3Node;

impl DecomposeVec3Node {
    pub const INPUT: &'static str = "vec";
    pub const OUTPUT_X: &'static str = "x";
    pub const OUTPUT_Y: &'static str = "y";
    pub const OUTPUT_Z: &'static str = "z";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for DecomposeVec3Node {
    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let Vec3 { x, y, z } = ctx.data_back(Self::INPUT)?.as_vec3()?;

        ctx.set_data_fwd(Self::OUTPUT_X, x);
        ctx.set_data_fwd(Self::OUTPUT_Y, y);
        ctx.set_data_fwd(Self::OUTPUT_Z, z);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::INPUT.into(), DataSpec::Vec3)].into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::OUTPUT_X.into(), DataSpec::F32),
            (Self::OUTPUT_Y.into(), DataSpec::F32),
            (Self::OUTPUT_Z.into(), DataSpec::F32),
        ]
        .into()
    }

    fn display_name(&self) -> String {
        "Decompose Vec3".into()
    }
}
