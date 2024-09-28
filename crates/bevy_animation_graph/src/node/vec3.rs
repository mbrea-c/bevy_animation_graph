use std::ops;

use bevy::prelude::*;

use crate::{
    core::{animation_graph::PinMap, errors::GraphError},
    node::math::{impl_clamp, impl_const, impl_for_1, impl_for_2, impl_lerp},
    prelude::*,
};

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
        //
        .register_type::<Length>()
        .register_type::<Normalize>()
        .register_type::<Dot>()
        //
        .register_type::<FromXyz>()
        .register_type::<IntoXyz>();
}

impl_const!(Const, Vec3, DataSpec::Vec3, "Vec3");
impl_for_2!(Add, Vec3, DataSpec::Vec3, "+ Add Vec3", ops::Add::add);
impl_for_2!(Sub, Vec3, DataSpec::Vec3, "- Sub Vec3", ops::Sub::sub);
impl_for_2!(Mul, Vec3, DataSpec::Vec3, "× Mul Vec3", ops::Mul::mul);
impl_for_2!(Div, Vec3, DataSpec::Vec3, "÷ Div Vec3", ops::Div::div);
impl_for_1!(Neg, Vec3, DataSpec::Vec3, "- Neg Vec3", ops::Neg::neg);
impl_for_1!(Abs, Vec3, DataSpec::Vec3, "|_| Abs Vec3", Vec3::abs);
impl_lerp!(Lerp, Vec3, DataSpec::Vec3, "Lerp Vec3", Vec3::lerp);
impl_clamp!(Clamp, Vec3, DataSpec::Vec3, "Clamp Vec3", Vec3::clamp);

impl_for_1!(
    Normalize,
    Vec3,
    DataSpec::Vec3,
    "Normalize Vec3",
    Vec3::normalize_or_zero
);
impl_for_2!(Dot, Vec3, DataSpec::Vec3, "⋅ Dot Vec3", Vec3::dot);

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct FromXyz;

impl FromXyz {
    pub const X: &str = "x";
    pub const Y: &str = "y";
    pub const Z: &str = "z";
    pub const VEC: &str = "vec";
}

impl NodeLike for FromXyz {
    fn display_name(&self) -> String {
        "Vec3 from XYZ".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let x = f32::try_from(ctx.data_back(Self::X)?).unwrap();
        let y = f32::try_from(ctx.data_back(Self::Y)?).unwrap();
        let z = f32::try_from(ctx.data_back(Self::Z)?).unwrap();
        ctx.set_data_fwd(Self::VEC, Vec3::new(x, y, z));
        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::X.into(), DataSpec::F32),
            (Self::Y.into(), DataSpec::F32),
            (Self::Z.into(), DataSpec::F32),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::VEC.into(), DataSpec::Vec3)].into()
    }
}

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct IntoXyz;

impl IntoXyz {
    pub const VEC: &str = "in";
    pub const X: &str = "x";
    pub const Y: &str = "y";
    pub const Z: &str = "z";
}

impl NodeLike for IntoXyz {
    fn display_name(&self) -> String {
        "Vec3 into XYZ".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let Vec3 { x, y, z } = ctx.data_back(Self::VEC)?.try_into().unwrap();
        ctx.set_data_fwd(Self::X, x);
        ctx.set_data_fwd(Self::Y, y);
        ctx.set_data_fwd(Self::Z, z);
        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::VEC.into(), DataSpec::Vec3)].into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::X.into(), DataSpec::F32),
            (Self::Y.into(), DataSpec::F32),
            (Self::Z.into(), DataSpec::F32),
        ]
        .into()
    }
}

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct Length;

impl Length {
    pub const IN: &str = "in";
    pub const OUT: &str = "out";
}

impl NodeLike for Length {
    fn display_name(&self) -> String {
        "Length Vec3".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let v = Vec3::try_from(ctx.data_back(Self::IN)?).unwrap();
        ctx.set_data_fwd(Self::OUT, v.length());
        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::IN.into(), DataSpec::Vec3)].into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::F32)].into()
    }
}
