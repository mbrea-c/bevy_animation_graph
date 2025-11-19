use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::DataSpec;
use crate::core::errors::GraphError;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct ConstF32 {
    pub constant: f32,
}

impl ConstF32 {
    pub const OUTPUT: &'static str = "out";

    pub fn new(constant: f32) -> Self {
        Self { constant }
    }
}

impl NodeLike for ConstF32 {
    fn display_name(&self) -> String {
        "F32".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        ctx.set_data_fwd(Self::OUTPUT, self.constant);
        Ok(())
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::F32)].into()
    }
}
