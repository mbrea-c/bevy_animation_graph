use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::SpecContext;
use crate::prelude::new_context::NodeContext;
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

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::F32)].into()
    }
}
