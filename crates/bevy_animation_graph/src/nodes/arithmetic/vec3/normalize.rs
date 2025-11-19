use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::DataSpec;
use crate::core::errors::GraphError;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct NormalizeVec3Node;

impl NormalizeVec3Node {
    pub const INPUT: &'static str = "in";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for NormalizeVec3Node {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input: Vec3 = ctx.data_back(Self::INPUT)?.as_vec3()?;
        let output = input.normalize_or_zero();

        ctx.set_data_fwd(Self::OUTPUT, output);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::INPUT.into(), DataSpec::Vec3)].into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Vec3)].into()
    }

    fn display_name(&self) -> String {
        "Normalize Vec3".into()
    }
}
