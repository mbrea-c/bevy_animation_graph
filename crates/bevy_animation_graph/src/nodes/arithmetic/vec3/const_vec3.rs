use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::DataSpec;
use crate::core::errors::GraphError;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct ConstVec3Node {
    pub constant: Vec3,
}

impl ConstVec3Node {
    pub const OUTPUT: &'static str = "out";

    pub fn new(constant: Vec3) -> Self {
        Self { constant }
    }
}

impl NodeLike for ConstVec3Node {
    fn display_name(&self) -> String {
        "Vec3".into()
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        ctx.set_data_fwd(Self::OUTPUT, self.constant);
        Ok(())
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Vec3)].into()
    }
}
