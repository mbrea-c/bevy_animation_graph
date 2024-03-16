use crate::core::animation_graph::{PinId, PinMap, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::errors::GraphError;
use crate::core::pose::{Pose, PoseSpec};
use crate::interpolation::prelude::InterpolateLinear;
use crate::prelude::{ParamValue, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct LoopNode {
    pub interpolation_period: f32,
}

impl LoopNode {
    pub const INPUT: &'static str = "Pose In";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new(interpolation_period: f32) -> Self {
        Self {
            interpolation_period,
        }
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Loop(self))
    }
}

impl NodeLike for LoopNode {
    fn parameter_pass(&self, _: PassContext) -> Result<HashMap<PinId, ParamValue>, GraphError> {
        Ok(HashMap::new())
    }

    fn duration_pass(&self, _: PassContext) -> Result<Option<DurationData>, GraphError> {
        Ok(Some(None))
    }

    fn pose_pass(
        &self,
        input: TimeUpdate,
        mut ctx: PassContext,
    ) -> Result<Option<Pose>, GraphError> {
        let duration = ctx.duration_back(Self::INPUT)?;

        let Some(duration) = duration else {
            return Ok(Some(ctx.pose_back(Self::INPUT, input)?));
        };

        let full_duration = duration + self.interpolation_period;

        let prev_time = ctx.prev_time_fwd();
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

        let mut pose = ctx.pose_back(Self::INPUT, fw_upd)?;

        if t > duration && t < full_duration {
            let start_pose = ctx.uncached_pose_back(Self::INPUT, TimeUpdate::Absolute(0.))?;
            let old_time = pose.timestamp;
            pose = pose.interpolate_linear(&start_pose, (t - duration) / self.interpolation_period);
            pose.timestamp = old_time;
        }

        let t_extra = curr_time.div_euclid(full_duration) * full_duration;
        pose.timestamp += t_extra;

        Ok(Some(pose))
    }

    fn pose_input_spec(&self, _: SpecContext) -> PinMap<PoseSpec> {
        [(Self::INPUT.into(), PoseSpec::Any)].into()
    }

    fn pose_output_spec(&self, _: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::Any)
    }

    fn display_name(&self) -> String {
        "🔄 Loop".into()
    }
}
