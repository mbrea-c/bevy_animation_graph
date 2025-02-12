use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::interpolation::prelude::InterpolateLinear;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;

/// This node pads the duration of an animation with a configurable period where
/// the last frame interpolates to the first
#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct PaddingNode {
    pub interpolation_period: f32,
}

impl PaddingNode {
    pub const IN_POSE: &'static str = "pose";
    pub const IN_TIME: &'static str = "time";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(interpolation_period: f32) -> Self {
        Self {
            interpolation_period,
        }
    }
}

impl NodeLike for PaddingNode {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let duration = ctx
            .duration_back(Self::IN_TIME)?
            .map(|d| d + self.interpolation_period);
        ctx.set_duration_fwd(duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;

        ctx.set_time_update_back(Self::IN_TIME, input);
        let mut pose = ctx.data_back(Self::IN_POSE)?.into_pose().unwrap();
        ctx.set_time(pose.timestamp);

        let Some(duration) = ctx.duration_back(Self::IN_TIME)? else {
            ctx.set_data_fwd(Self::OUT_POSE, pose);
            return Ok(());
        };

        if pose.timestamp > duration {
            // Need to get initial pose and interpolate
            let mut ctx_temp = ctx.with_temp(true);
            ctx_temp.set_time_update_back(Self::IN_TIME, TimeUpdate::Absolute(0.));
            let start_pose = ctx_temp.data_back(Self::IN_POSE)?.into_pose().unwrap();
            // TODO: How to clear cache? time? pose?
            // ctx.clear_temp_cache(Self::IN_POSE);
            let old_time = pose.timestamp;
            let alpha = ((pose.timestamp - duration) / self.interpolation_period).clamp(0., 1.);
            pose = pose.interpolate_linear(&start_pose, alpha);
            pose.timestamp = old_time;
        }

        ctx.set_data_fwd(Self::OUT_POSE, pose);

        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::IN_POSE.into(), DataSpec::Pose)].into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        [(Self::IN_TIME.into(), ())].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "Padding".into()
    }
}
