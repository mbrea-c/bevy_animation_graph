use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
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
#[reflect(Default, NodeLike)]
pub struct CompareF32 {
    pub op: CompareOp,
}

impl CompareF32 {
    pub const INPUT_1: &'static str = "in_a";
    pub const INPUT_2: &'static str = "in_b";
    pub const OUTPUT: &'static str = "out";

    pub fn new(op: CompareOp) -> Self {
        Self { op }
    }
}

impl NodeLike for CompareF32 {
    fn display_name(&self) -> String {
        "== Compare".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input_1 = ctx.data_back(Self::INPUT_1)?.as_f32()?;
        let input_2 = ctx.data_back(Self::INPUT_2)?.as_f32()?;
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
