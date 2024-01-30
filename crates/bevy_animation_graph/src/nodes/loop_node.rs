use crate::core::animation_graph::{PinId, PinMap, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::frame::{PoseFrame, PoseSpec};
use crate::prelude::{ParamValue, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
pub struct LoopNode {}

impl LoopNode {
    pub const INPUT: &'static str = "Pose In";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Loop(self))
    }
}

impl NodeLike for LoopNode {
    fn parameter_pass(&self, _: PassContext) -> HashMap<PinId, ParamValue> {
        HashMap::new()
    }

    fn duration_pass(&self, _: PassContext) -> Option<DurationData> {
        Some(None)
    }

    fn pose_pass(&self, input: TimeUpdate, mut ctx: PassContext) -> Option<PoseFrame> {
        let duration = ctx.duration_back(Self::INPUT);

        let Some(duration) = duration else {
            return Some(ctx.pose_back(Self::INPUT, input));
        };

        let prev_time = ctx.prev_time_fwd();
        let curr_time = input.apply(prev_time);
        let t = curr_time.rem_euclid(duration);

        let fw_upd = match input {
            TimeUpdate::Delta(dt) => {
                if prev_time.div_euclid(duration) != curr_time.div_euclid(duration) {
                    TimeUpdate::Absolute(t)
                } else {
                    TimeUpdate::Delta(dt)
                }
            }
            TimeUpdate::Absolute(_) => TimeUpdate::Absolute(t),
        };

        let mut pose = ctx.pose_back(Self::INPUT, fw_upd);

        let t_extra = curr_time.div_euclid(duration) * duration;
        pose.map_ts(|t| t + t_extra);

        Some(pose)
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
