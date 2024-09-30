use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct BuildVec3Node;

impl BuildVec3Node {
    pub const INPUT_X: &'static str = "x";
    pub const INPUT_Y: &'static str = "y";
    pub const INPUT_Z: &'static str = "z";
    pub const OUTPUT: &'static str = "vec";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for BuildVec3Node {
    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let x: f32 = ctx.data_back(Self::INPUT_X)?.val();
        let y: f32 = ctx.data_back(Self::INPUT_Y)?.val();
        let z: f32 = ctx.data_back(Self::INPUT_Z)?.val();

        ctx.set_data_fwd(Self::OUTPUT, Vec3::new(x, y, z));

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT_X.into(), DataSpec::F32),
            (Self::INPUT_Y.into(), DataSpec::F32),
            (Self::INPUT_Z.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Vec3)].into()
    }

    fn display_name(&self) -> String {
        "Build Vec3".into()
    }
}
