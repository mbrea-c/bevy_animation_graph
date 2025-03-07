use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::prelude::DataSpec;
use crate::interpolation::prelude::InterpolateLinear;
use crate::prelude::{PassContext, SpecContext};
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct LoopNode {
    pub interpolation_period: f32,
}

impl LoopNode {
    pub const IN_POSE: &'static str = "pose";
    pub const IN_TIME: &'static str = "time";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(interpolation_period: f32) -> Self {
        Self {
            interpolation_period,
        }
    }
}

impl NodeLike for LoopNode {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        ctx.set_duration_fwd(None);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        let duration = ctx.duration_back(Self::IN_TIME)?;

        let Some(duration) = duration else {
            ctx.set_time_update_back(Self::IN_TIME, input);
            let pose_back = ctx.data_back(Self::IN_POSE)?.into_pose().unwrap();
            ctx.set_time(pose_back.timestamp);
            ctx.set_data_fwd(Self::OUT_POSE, pose_back);

            return Ok(());
        };

        let full_duration = duration + self.interpolation_period;

        let prev_time = ctx.prev_time();

        let (curr_time, t, fw_upd) = match input {
            TimeUpdate::Delta(dt) => {
                let curr_time = prev_time + dt;
                let t = curr_time.rem_euclid(full_duration);

                let fw_upd =
                    if prev_time.div_euclid(full_duration) != curr_time.div_euclid(full_duration) {
                        TimeUpdate::Absolute(t)
                    } else {
                        TimeUpdate::Delta(dt)
                    };

                (curr_time, t, fw_upd)
            }
            TimeUpdate::Absolute(curr_time) => {
                let t = curr_time.rem_euclid(full_duration);
                (curr_time, t, TimeUpdate::Absolute(t))
            }
            TimeUpdate::PercentOfEvent { .. } => {
                todo!("we probably want to return here and issue a warning")
            }
        };

        ctx.set_time_update_back(Self::IN_TIME, fw_upd);
        let mut pose = ctx.data_back(Self::IN_POSE)?.into_pose().unwrap();

        if t > duration && t < full_duration {
            let mut ctx_temp = ctx.with_temp(true);
            ctx_temp.set_time_update_back(Self::IN_TIME, TimeUpdate::Absolute(0.));
            let start_pose = ctx_temp.data_back(Self::IN_POSE)?.into_pose().unwrap();
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
        "🔄 Loop".into()
    }
}
