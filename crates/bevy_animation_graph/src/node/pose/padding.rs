use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::pose::Pose;
use crate::core::prelude::DataSpec;
use crate::interpolation::prelude::InterpolateLinear;
use crate::prelude::{PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;

/// This node pads the duration of an animation with a configurable period where
/// the last frame interpolates to the first
#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::node::pose"]
pub struct Padding {
    pub interpolation_period: f32,
}

impl Padding {
    pub const IN_POSE: &str = "in_pose";
    pub const IN_TIME: &str = "in_time";
    pub const OUT_POSE: &str = "pose";
}

impl NodeLike for Padding {
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
        let mut pose: Pose = ctx.data_back(Self::IN_POSE)?.val();
        ctx.set_time(pose.timestamp);

        let Some(duration) = ctx.duration_back(Self::IN_TIME)? else {
            ctx.set_data_fwd(Self::OUT_POSE, pose);
            return Ok(());
        };

        if pose.timestamp > duration {
            // Need to get initial pose and interpolate
            let mut ctx_temp = ctx.with_temp(true);
            ctx_temp.set_time_update_back(Self::IN_TIME, TimeUpdate::Absolute(0.));
            let start_pose: Pose = ctx_temp.data_back(Self::IN_POSE)?.val();
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
