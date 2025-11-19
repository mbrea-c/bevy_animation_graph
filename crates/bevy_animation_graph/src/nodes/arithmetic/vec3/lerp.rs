use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::DataSpec;
use crate::core::errors::GraphError;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct LerpVec3Node;

impl LerpVec3Node {
    pub const INPUT_A: &'static str = "a";
    pub const INPUT_B: &'static str = "b";
    pub const INPUT_FACTOR: &'static str = "factor";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for LerpVec3Node {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let a: Vec3 = ctx.data_back(Self::INPUT_A)?.as_vec3()?;
        let b: Vec3 = ctx.data_back(Self::INPUT_B)?.as_vec3()?;
        let factor: f32 = ctx.data_back(Self::INPUT_FACTOR)?.as_f32()?;

        let output = Vec3::lerp(a, b, factor);

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT_A.into(), DataSpec::Vec3),
            (Self::INPUT_B.into(), DataSpec::Vec3),
            (Self::INPUT_FACTOR.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Vec3)].into()
    }

    fn display_name(&self) -> String {
        "Lerp Vec3".into()
    }
}
