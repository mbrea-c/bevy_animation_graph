use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
};
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
#[type_path = "bevy_animation_graph::builtin_nodes"]
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

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
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

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::INPUT_1, DataSpec::F32)
            .add_input_data(Self::INPUT_2, DataSpec::F32)
            .add_output_data(Self::OUTPUT, DataSpec::F32);

        Ok(())
    }
}
