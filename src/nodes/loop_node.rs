use crate::core::animation_graph::{
    OptParamSpec, ParamSpec, ParamValue, PinId, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::frame::PoseFrame;
use crate::prelude::{DurationData, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Reflect, Clone, Debug, Default)]
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
    fn parameter_pass(
        &self,
        _inputs: HashMap<PinId, ParamValue>,
        _: PassContext,
    ) -> HashMap<PinId, ParamValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        _inputs: HashMap<PinId, Option<f32>>,
        _: PassContext,
    ) -> Option<DurationData> {
        Some(None)
    }

    fn time_pass(&self, input: TimeState, ctx: PassContext) -> HashMap<PinId, TimeUpdate> {
        let duration = ctx.duration_back(Self::INPUT);

        let Some(duration) = duration else {
            return HashMap::from([(Self::INPUT.into(), input.update)]);
        };

        let t = input.time.rem_euclid(duration);

        let fw_upd = match input.update {
            TimeUpdate::Delta(dt) => {
                let prev_time = input.time - dt;
                if prev_time.div_euclid(duration) != input.time.div_euclid(duration) {
                    TimeUpdate::Absolute(t)
                } else {
                    TimeUpdate::Delta(dt)
                }
            }
            TimeUpdate::Absolute(_) => TimeUpdate::Absolute(t),
        };

        HashMap::from([(Self::INPUT.into(), fw_upd)])
    }

    fn time_dependent_pass(
        &self,
        mut inputs: HashMap<PinId, PoseFrame>,
        ctx: PassContext,
    ) -> Option<PoseFrame> {
        let time = ctx.time_fwd().time;
        let duration = ctx.duration_back(Self::INPUT);

        let mut in_pose_frame = inputs.remove(Self::INPUT).unwrap();

        if let Some(duration) = duration {
            let t_extra = time.div_euclid(duration) * duration;
            in_pose_frame.map_ts(|t| t + t_extra);
        }

        Some(in_pose_frame)
    }

    fn parameter_input_spec(&self, _: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self, _: SpecContext) -> HashMap<PinId, ParamSpec> {
        HashMap::new()
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashSet<PinId> {
        HashSet::from([Self::INPUT.into()])
    }

    fn pose_output_spec(&self, _: SpecContext) -> bool {
        true
    }

    fn display_name(&self) -> String {
        "🗘 Loop".into()
    }
}
