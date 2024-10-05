macro_rules! impl_const {
    ($struct:ident, $value_ty:ty, $value_spec:expr, $name:expr) => {
        #[derive(Reflect, Clone, Debug, Default)]
        #[reflect(Default, NodeLike)]
        pub struct $struct(pub $value_ty);

        impl $struct {
            pub const IN: &str = "in";
            pub const OUT: &str = "out";
        }

        impl NodeLike for $struct {
            fn display_name(&self) -> String {
                $name.into()
            }

            fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
                ctx.set_data_fwd(Self::OUT, self.0.clone());
                Ok(())
            }

            fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
                [].into()
            }

            fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
                [(Self::OUT.into(), $value_spec)].into()
            }
        }
    };
}

pub(crate) use impl_const;

macro_rules! impl_for_1 {
    ($struct:ident, $value_ty:ty, $value_spec:expr, $name:expr, $op:expr) => {
        #[derive(Reflect, Clone, Debug, Default)]
        #[reflect(Default, NodeLike)]
        pub struct $struct;

        impl $struct {
            pub const IN: &str = "in";
            pub const OUT: &str = "out";
        }

        impl NodeLike for $struct {
            fn display_name(&self) -> String {
                $name.into()
            }

            fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
                let v = <$value_ty>::try_from(ctx.data_back(Self::IN)?).unwrap();
                ctx.set_data_fwd(Self::OUT, ($op)(v));
                Ok(())
            }

            fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
                [(Self::IN.into(), $value_spec)].into()
            }

            fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
                [(Self::OUT.into(), $value_spec)].into()
            }
        }
    };
}

pub(crate) use impl_for_1;

macro_rules! impl_for_2 {
    ($struct:ident, $value_ty:ty, $value_spec:expr, $name:expr, $op:expr) => {
        #[derive(Reflect, Clone, Debug, Default)]
        #[reflect(Default, NodeLike)]
        pub struct $struct;

        impl $struct {
            pub const IN_A: &str = "in_a";
            pub const IN_B: &str = "in_b";
            pub const OUT: &str = "out";
        }

        impl NodeLike for $struct {
            fn display_name(&self) -> String {
                $name.into()
            }

            fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
                let a = <$value_ty>::try_from(ctx.data_back(Self::IN_A)?).unwrap();
                let b = <$value_ty>::try_from(ctx.data_back(Self::IN_B)?).unwrap();
                ctx.set_data_fwd(Self::OUT, ($op)(a, b));
                Ok(())
            }

            fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
                [
                    (Self::IN_A.into(), $value_spec),
                    (Self::IN_B.into(), $value_spec),
                ]
                .into()
            }

            fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
                [(Self::OUT.into(), $value_spec)].into()
            }
        }
    };
}

pub(crate) use impl_for_2;

macro_rules! impl_lerp {
    ($struct:ident, $value_ty:ty, $value_spec:expr, $name:expr, $op:expr) => {
        #[derive(Reflect, Clone, Debug, Default)]
        #[reflect(Default, NodeLike)]
        pub struct $struct;

        impl $struct {
            pub const FROM: &str = "from";
            pub const TO: &str = "to";
            pub const FACTOR: &str = "factor";
            pub const OUT: &str = "out";
        }

        impl NodeLike for $struct {
            fn display_name(&self) -> String {
                $name.into()
            }

            fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
                let from = <$value_ty>::try_from(ctx.data_back(Self::FROM)?).unwrap();
                let to = <$value_ty>::try_from(ctx.data_back(Self::TO)?).unwrap();
                let fac = f32::try_from(ctx.data_back(Self::FACTOR)?).unwrap();
                ctx.set_data_fwd(Self::OUT, ($op)(from, to, fac));
                Ok(())
            }

            fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
                [
                    (Self::FROM.into(), $value_spec),
                    (Self::TO.into(), $value_spec),
                    (Self::FACTOR.into(), DataSpec::F32),
                ]
                .into()
            }

            fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
                [(Self::OUT.into(), $value_spec)].into()
            }
        }
    };
}

pub(crate) use impl_lerp;

macro_rules! impl_clamp {
    ($struct:ident, $value_ty:ty, $value_spec:expr, $name:expr, $op:expr) => {
        #[derive(Reflect, Clone, Debug, Default)]
        #[reflect(Default, NodeLike)]
        pub struct $struct;

        impl $struct {
            pub const IN: &str = "in";
            pub const MIN: &str = "min";
            pub const MAX: &str = "max";
            pub const OUT: &str = "out";
        }

        impl NodeLike for $struct {
            fn display_name(&self) -> String {
                $name.into()
            }

            fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
                let v = <$value_ty>::try_from(ctx.data_back(Self::IN)?).unwrap();
                let min = <$value_ty>::try_from(ctx.data_back(Self::MIN)?).unwrap();
                let max = <$value_ty>::try_from(ctx.data_back(Self::MAX)?).unwrap();
                ctx.set_data_fwd(Self::OUT, ($op)(v, min, max));
                Ok(())
            }

            fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
                [
                    (Self::IN.into(), $value_spec),
                    (Self::MIN.into(), $value_spec),
                    (Self::MAX.into(), $value_spec),
                ]
                .into()
            }

            fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
                [(Self::OUT.into(), $value_spec)].into()
            }
        }
    };
}

pub(crate) use impl_clamp;
