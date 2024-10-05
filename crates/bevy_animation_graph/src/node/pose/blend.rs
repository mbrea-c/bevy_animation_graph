use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::pose::Pose;
use crate::core::prelude::DataSpec;
use crate::prelude::{InterpolateLinear, PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[reflect(Default, Serialize)]
pub enum BlendMode {
    #[default]
    LinearInterpolate,
    Additive,
    Difference,
}

#[derive(Reflect, Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[reflect(Default)]
pub enum BlendSyncMode {
    /// Sets the absolute timestamp of input 2 equal to the timestamp from input 1
    #[default]
    Absolute,
    /// Propagates the same time update that was received, does not try to sync the inputs.
    NoSync,
}

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::node::pose"]
pub struct Blend {
    pub mode: BlendMode,
    pub sync_mode: BlendSyncMode,
}

impl Blend {
    pub const FACTOR: &str = "factor";
    pub const IN_POSE_A: &str = "in_pose_a";
    pub const IN_TIME_A: &str = "in_time_a";
    pub const IN_POSE_B: &str = "in_pose_b";
    pub const IN_TIME_B: &str = "in_time_b";
    pub const OUT: &str = "out";
}

impl NodeLike for Blend {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let duration_1 = ctx.duration_back(Self::IN_TIME_A)?;
        let duration_2 = ctx.duration_back(Self::IN_TIME_B)?;

        let out_duration = match (duration_1, duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1.max(duration_2)),
            (Some(duration_1), None) => Some(duration_1),
            (None, Some(duration_2)) => Some(duration_2),
            (None, None) => None,
        };

        ctx.set_duration_fwd(out_duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;

        ctx.set_time_update_back(Self::IN_TIME_A, input);
        let in_frame_1: Pose = ctx.data_back(Self::IN_POSE_A)?.val();

        match self.sync_mode {
            BlendSyncMode::Absolute => {
                ctx.set_time_update_back(
                    Self::IN_TIME_B,
                    TimeUpdate::Absolute(in_frame_1.timestamp),
                );
            }
            BlendSyncMode::NoSync => {
                ctx.set_time_update_back(Self::IN_TIME_B, input);
            }
        };

        let in_frame_2: Pose = ctx.data_back(Self::IN_POSE_B)?.val();

        let out = match self.mode {
            BlendMode::LinearInterpolate => {
                let alpha = ctx.data_back(Self::FACTOR)?.unwrap_f32();
                in_frame_1.interpolate_linear(&in_frame_2, alpha)
            }
            BlendMode::Additive => {
                let alpha = ctx.data_back(Self::FACTOR)?.unwrap_f32();
                in_frame_1.additive_blend(&in_frame_2, alpha)
            }
            BlendMode::Difference => in_frame_1.difference(&in_frame_2),
        };

        ctx.set_time(out.timestamp);
        ctx.set_data_fwd(Self::OUT, out);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::FACTOR.into(), DataSpec::F32),
            (Self::IN_POSE_A.into(), DataSpec::Pose),
            (Self::IN_POSE_B.into(), DataSpec::Pose),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        [(Self::IN_TIME_A.into(), ()), (Self::IN_TIME_B.into(), ())].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "∑ Blend".into()
    }
}
