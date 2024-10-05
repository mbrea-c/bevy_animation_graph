use std::ops;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::node::math::{impl_clamp, impl_const, impl_for_1, impl_for_2, impl_lerp};
use crate::prelude::{PassContext, SpecContext};

pub(super) fn register_types(app: &mut App) {
    app.register_type::<Const>()
        .register_type::<Add>()
        .register_type::<Sub>()
        .register_type::<Mul>()
        .register_type::<Div>()
        .register_type::<Neg>()
        .register_type::<Abs>()
        .register_type::<Lerp>()
        .register_type::<Clamp>()
        .register_type::<Compare>()
        .register_type::<Select>();
}

impl_const!(Const, f32, DataSpec::F32, "F32");
impl_for_2!(Add, f32, DataSpec::F32, "+ Add F32", ops::Add::add);
impl_for_2!(Sub, f32, DataSpec::F32, "- Sub F32", ops::Sub::sub);
impl_for_2!(Mul, f32, DataSpec::F32, "× Mul F32", ops::Mul::mul);
impl_for_2!(Div, f32, DataSpec::F32, "÷ Div F32", ops::Div::div);
impl_for_1!(Neg, f32, DataSpec::F32, "- Neg F32", ops::Neg::neg);
impl_for_1!(Abs, f32, DataSpec::F32, "|_| Abs F32", f32::abs);
impl_lerp!(Lerp, f32, DataSpec::F32, "Lerp F32", f32::lerp);
impl_clamp!(Clamp, f32, DataSpec::F32, "Clamp F32", f32::clamp);

#[derive(Reflect, Clone, Copy, Default, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CompareOp {
    Less,
    LessEqual,
    More,
    #[default]
    MoreEqual,
}

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct Compare {
    pub op: CompareOp,
}

impl Compare {
    pub const IN_A: &str = "in_a";
    pub const IN_B: &str = "in_b";
    pub const OUT: &str = "out";
}

impl NodeLike for Compare {
    fn display_name(&self) -> String {
        let op_symbol = match self.op {
            CompareOp::Less => "<",
            CompareOp::LessEqual => "≤",
            CompareOp::More => ">",
            CompareOp::MoreEqual => "≥",
        };
        format!("{op_symbol} Compare F32").into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let a = f32::try_from(ctx.data_back(Self::IN_A)?).unwrap();
        let b = f32::try_from(ctx.data_back(Self::IN_B)?).unwrap();
        ctx.set_data_fwd(
            Self::OUT,
            match self.op {
                CompareOp::Less => a < b,
                CompareOp::LessEqual => a <= b,
                CompareOp::More => a > b,
                CompareOp::MoreEqual => a >= b,
            },
        );
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::IN_A.into(), DataSpec::F32),
            (Self::IN_B.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::Bool)].into()
    }
}

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct Select;

impl Select {
    pub const CONDITION: &'static str = "condition";
    pub const IF_FALSE: &'static str = "if_false";
    pub const IF_TRUE: &'static str = "if_true";
    pub const OUT: &'static str = "out";
}

impl NodeLike for Select {
    fn display_name(&self) -> String {
        "Select F32".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let condition = bool::try_from(ctx.data_back(Self::CONDITION)?).unwrap();
        let if_false = f32::try_from(ctx.data_back(Self::IF_FALSE)?).unwrap();
        let if_true = f32::try_from(ctx.data_back(Self::IF_TRUE)?).unwrap();
        ctx.set_data_fwd(Self::OUT, if condition { if_true } else { if_false });
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::CONDITION.into(), DataSpec::Bool),
            (Self::IF_FALSE.into(), DataSpec::F32),
            (Self::IF_TRUE.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::F32)].into()
    }
}
