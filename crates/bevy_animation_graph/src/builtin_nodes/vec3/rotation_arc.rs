use crate::core::animation_graph::PinMap;
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::DataSpec;
use crate::core::errors::GraphError;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct RotationArcNode;

impl RotationArcNode {
    pub const INPUT_1: &'static str = "in_a";
    pub const INPUT_2: &'static str = "in_b";
    pub const OUTPUT: &'static str = "out";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for RotationArcNode {
    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input_1: Vec3 = ctx.data_back(Self::INPUT_1)?.as_vec3()?;
        let input_2: Vec3 = ctx.data_back(Self::INPUT_2)?.as_vec3()?;

        ctx.set_data_fwd(Self::OUTPUT, Quat::from_rotation_arc(input_1, input_2));

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::INPUT_1.into(), DataSpec::Vec3),
            (Self::INPUT_2.into(), DataSpec::Vec3),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUTPUT.into(), DataSpec::Quat)].into()
    }

    fn display_name(&self) -> String {
        "Rotation Arc".into()
    }
}
