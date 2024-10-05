use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::pose::Pose;
use crate::core::prelude::DataSpec;
use crate::interpolation::prelude::InterpolateLinear;
use crate::prelude::{PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::node::pose"]
pub struct Repeat {
    pub interpolation_period: f32,
}

impl Repeat {
    pub const IN_POSE: &str = "in_pose";
    pub const IN_TIME: &str = "in_time";
    pub const OUT: &str = "out";
}

impl NodeLike for Repeat {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        ctx.set_duration_fwd(None);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        let duration = ctx.duration_back(Self::IN_TIME)?;

        let Some(duration) = duration else {
            ctx.set_time_update_back(Self::IN_TIME, input);
            let pose_back: Pose = ctx.data_back(Self::IN_POSE)?.val();
            ctx.set_time(pose_back.timestamp);
            ctx.set_data_fwd(Self::OUT, pose_back);

            return Ok(());
        };

        let full_duration = duration + self.interpolation_period;

        let prev_time = ctx.prev_time();
        let curr_time = input.apply(prev_time);
        let t = curr_time.rem_euclid(full_duration);

        let fw_upd = match input {
            TimeUpdate::Delta(dt) => {
                if prev_time.div_euclid(full_duration) != curr_time.div_euclid(full_duration) {
                    TimeUpdate::Absolute(t)
                } else {
                    TimeUpdate::Delta(dt)
                }
            }
            TimeUpdate::Absolute(_) => TimeUpdate::Absolute(t),
        };

        ctx.set_time_update_back(Self::IN_TIME, fw_upd);
        let mut pose: Pose = ctx.data_back(Self::IN_POSE)?.val();

        if t > duration && t < full_duration {
            let mut ctx_temp = ctx.with_temp(true);
            ctx_temp.set_time_update_back(Self::IN_TIME, TimeUpdate::Absolute(0.));
            let start_pose: Pose = ctx_temp.data_back(Self::IN_POSE)?.val();
            // TODO: How to clear cache? time? pose?
            // ctx.clear_temp_cache(Self::IN_POSE);
            let old_time = pose.timestamp;
            let alpha = (t - duration) / self.interpolation_period;
            pose = pose.interpolate_linear(&start_pose, alpha);
            pose.timestamp = old_time;
        }

        let t_extra = curr_time.div_euclid(full_duration) * full_duration;
        pose.timestamp += t_extra;
        ctx.set_time(pose.timestamp);
        ctx.set_data_fwd(Self::OUT, pose);

        Ok(())
    }

    fn data_input_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::IN_POSE.into(), DataSpec::Pose)].into()
    }

    fn data_output_spec(&self, _ctx: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        [(Self::IN_TIME.into(), ())].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "🔄 Loop".into()
    }
}
