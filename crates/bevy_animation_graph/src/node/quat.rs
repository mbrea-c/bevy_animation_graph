use std::ops;

use bevy::prelude::*;

use crate::{
    core::{animation_graph::PinMap, errors::GraphError},
    node::math::{impl_const, impl_for_1, impl_for_2, impl_lerp},
    prelude::*,
};

use super::*;

impl_const!(Const, Quat, DataSpec::Quat, "Quat");
impl_for_1!(Invert, Quat, DataSpec::Quat, "Invert Quat", Quat::inverse);
impl_for_2!(Mul, Quat, DataSpec::Quat, "× Mul Quat", ops::Mul::mul);
impl_lerp!(Slerp, Quat, DataSpec::Quat, "Slerp Quat", Quat::slerp);

pub(super) fn register_types(app: &mut App) {
    app.register_type::<Const>()
        .register_type::<Invert>()
        .register_type::<Mul>()
        .register_type::<Slerp>()
        //
        .register_type::<FromEuler>()
        .register_type::<IntoEuler>()
        .register_type::<FromRotationArc>();
}

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct FromEuler {
    pub mode: EulerRot,
}

impl FromEuler {
    pub const EULER: &'static str = "euler";
    pub const QUAT: &'static str = "out";
}

impl NodeLike for FromEuler {
    fn display_name(&self) -> String {
        "Quat from Euler".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let Vec3 { x, y, z } = ctx.data_back(Self::EULER)?.try_into().unwrap();
        ctx.set_data_fwd(Self::QUAT, Quat::from_euler(self.mode, x, y, z));
        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::EULER.into(), DataSpec::Vec3)].into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::QUAT.into(), DataSpec::Quat)].into()
    }
}

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct IntoEuler {
    pub mode: EulerRot,
}

impl IntoEuler {
    pub const QUAT: &'static str = "quat";
    pub const EULER: &'static str = "euler";
}

impl NodeLike for IntoEuler {
    fn display_name(&self) -> String {
        "Quat into Euler".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let quat = Quat::try_from(ctx.data_back(Self::QUAT)?).unwrap();
        let (x, y, z) = quat.to_euler(self.mode);
        ctx.set_data_fwd(Self::EULER, Vec3::new(x, y, z));
        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::QUAT.into(), DataSpec::Quat)].into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::EULER.into(), DataSpec::Vec3)].into()
    }
}

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct FromRotationArc;

impl FromRotationArc {
    pub const IN_A: &'static str = "in_a";
    pub const IN_B: &'static str = "in_b";
    pub const OUT: &'static str = "out";
}

impl NodeLike for FromRotationArc {
    fn display_name(&self) -> String {
        "Quat from Rotation Arc".into()
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let a = Vec3::try_from(ctx.data_back(Self::IN_A)?).unwrap();
        let b = Vec3::try_from(ctx.data_back(Self::IN_B)?).unwrap();
        ctx.set_data_fwd(Self::OUT, Quat::from_rotation_arc(a, b));
        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::IN_A.into(), DataSpec::Vec3),
            (Self::IN_B.into(), DataSpec::Vec3),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::Quat)].into()
    }
}
