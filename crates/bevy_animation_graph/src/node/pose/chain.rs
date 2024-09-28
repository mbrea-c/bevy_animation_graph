use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::pose::Pose;
use crate::core::prelude::DataSpec;
use crate::prelude::{InterpolateLinear, PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::node::pose"]
pub struct Chain {
    /// Time in-between animations where the output should interpolate between the last pose of the
    /// first animation and the first pose of the second
    pub interpolation_period: f32,
}

impl Chain {
    pub const POSE_A: &str = "pose_a";
    pub const TIME_A: &str = "time_a";
    pub const POSE_B: &str = "pose_b";
    pub const TIME_B: &str = "time_b";
    pub const OUT: &str = "out";

    pub fn new(interpolation_period: f32) -> Self {
        Self {
            interpolation_period,
        }
    }
}

impl NodeLike for Chain {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let source_duration_1 = ctx.duration_back(Self::TIME_A)?;
        let source_duration_2 = ctx.duration_back(Self::TIME_B)?;

        let out_duration = match (source_duration_1, source_duration_2) {
            (Some(duration_1), Some(duration_2)) => {
                Some(duration_1 + duration_2 + self.interpolation_period)
            }
            (Some(_), None) => None,
            (None, Some(_)) => None,
            (None, None) => None,
        };

        ctx.set_duration_fwd(out_duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        let duration_1 = ctx.duration_back(Self::TIME_A)?;
        let Some(duration_1) = duration_1 else {
            // First input is infinite, forward time update without change
            ctx.set_time_update_back(Self::TIME_A, input);
            let pose_a: Pose = ctx.data_back(Self::POSE_A)?.val();
            ctx.set_time(pose_a.timestamp);
            ctx.set_data_fwd(Self::OUT, pose_a);
            return Ok(());
        };
        ctx.set_time_update_back(Self::TIME_A, input);
        let pose_a: Pose = ctx.data_back(Self::POSE_A)?.val();
        let curr_time = pose_a.timestamp;
        ctx.set_time(curr_time);

        if curr_time < duration_1 {
            ctx.set_data_fwd(Self::OUT, pose_a);
        } else if curr_time - duration_1 - self.interpolation_period >= 0. {
            ctx.set_time_update_back(
                Self::TIME_B,
                TimeUpdate::Absolute(curr_time - duration_1 - self.interpolation_period),
            );
            let mut pose_b: Pose = ctx.data_back(Self::POSE_B)?.val();
            pose_b.timestamp = curr_time;
            ctx.set_data_fwd(Self::OUT, pose_b);
        } else {
            ctx.set_time_update_back(Self::TIME_B, TimeUpdate::Absolute(0.0));
            let pose_2: Pose = ctx.data_back(Self::POSE_B)?.val();
            let mut out_pose = pose_a.interpolate_linear(
                &pose_2,
                (curr_time - duration_1) / self.interpolation_period,
            );
            out_pose.timestamp = curr_time;
            ctx.set_data_fwd(Self::OUT, out_pose);
        }

        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::POSE_A.into(), DataSpec::Pose),
            (Self::POSE_B.into(), DataSpec::Pose),
        ]
        .into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _ctx: SpecContext) -> PinMap<()> {
        [(Self::TIME_A.into(), ()), (Self::TIME_B.into(), ())].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "⛓ Chain".into()
    }
}
