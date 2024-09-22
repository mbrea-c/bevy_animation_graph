use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct ConstF32 {
    pub constant: f32,
}

impl ConstF32 {
    pub const OUTPUT: &'static str = "out";

    pub fn new(constant: f32) -> Self {
        Self { constant }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::ConstF32(self))
    }
}

impl NodeLike for ConstF32 {
    fn display_name(&self) -> String {
        "F32".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        ctx.set_data_fwd(Self::OUTPUT, self.constant);
        Ok(())
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::F32)].into()
    }
}
