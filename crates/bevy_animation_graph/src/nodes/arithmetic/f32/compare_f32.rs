use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Copy, Default, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CompareOp {
    Less,
    LessEqual,
    More,
    MoreEqual,
    #[default]
    Equal,
}

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct CompareF32 {
    pub op: CompareOp,
}

impl CompareF32 {
    pub const INPUT_1: &'static str = "F32 In 1";
    pub const INPUT_2: &'static str = "F32 In 2";
    pub const OUTPUT: &'static str = "Bool Out";

    pub fn new(op: CompareOp) -> Self {
        Self { op }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::CompareF32(self))
    }
}

impl NodeLike for CompareF32 {
    fn display_name(&self) -> String {
        "== Compare".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input_1 = ctx.data_back(Self::INPUT_1)?.unwrap_f32();
        let input_2 = ctx.data_back(Self::INPUT_2)?.unwrap_f32();
        ctx.set_data_fwd(
            Self::OUTPUT,
            match self.op {
                CompareOp::Less => input_1 < input_2,
                CompareOp::LessEqual => input_1 <= input_2,
                CompareOp::More => input_1 > input_2,
                CompareOp::MoreEqual => input_1 >= input_2,
                CompareOp::Equal => input_1 == input_2,
            },
        );
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT_1.into(), DataSpec::F32),
            (Self::INPUT_2.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Bool)].into()
    }
}
