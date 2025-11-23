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
pub struct ClampF32;

impl ClampF32 {
    pub const INPUT: &'static str = "in";
    pub const CLAMP_MIN: &'static str = "min";
    pub const CLAMP_MAX: &'static str = "max";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for ClampF32 {
    fn display_name(&self) -> String {
        "Clamp".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.data_back(Self::INPUT)?.as_f32()?;
        let min = ctx.data_back(Self::CLAMP_MIN)?.as_f32()?;
        let max = ctx.data_back(Self::CLAMP_MAX)?.as_f32()?;
        ctx.set_data_fwd(Self::OUTPUT, input.clamp(min, max));
        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT, DataSpec::F32)
            .add_input_data(Self::CLAMP_MIN, DataSpec::F32)
            .add_input_data(Self::CLAMP_MAX, DataSpec::F32)
            .add_output_data(Self::OUTPUT, DataSpec::F32);
        Ok(())
    }
}
