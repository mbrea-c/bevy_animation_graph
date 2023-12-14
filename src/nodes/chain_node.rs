use crate::chaining::Chainable;
use crate::core::animation_graph::{
    OptParamSpec, ParamSpec, ParamValue, PinId, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::frame::PoseFrame;
use crate::prelude::{DurationData, PassContext, SpecContext};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Reflect, Clone, Debug, Default)]
pub struct ChainNode {}

impl ChainNode {
    pub const INPUT_1: &'static str = "Pose In 1";
    pub const INPUT_2: &'static str = "Pose In 2";
    pub const OUTPUT: &'static str = "Pose Out";

    pub fn new() -> Self {
        Self {}
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Chain(self))
    }
}

impl NodeLike for ChainNode {
    fn parameter_pass(
        &self,
        _inputs: HashMap<PinId, ParamValue>,
        _: PassContext,
    ) -> HashMap<PinId, ParamValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<PinId, Option<f32>>,
        _: PassContext,
    ) -> Option<DurationData> {
        let source_duration_1 = *inputs.get(Self::INPUT_1).unwrap();
        let source_duration_2 = *inputs.get(Self::INPUT_2).unwrap();

        let out_duration = match (source_duration_1, source_duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1 + duration_2),
            (Some(_), None) => None,
            (None, Some(_)) => None,
            (None, None) => None,
        };

        Some(out_duration)
    }

    fn time_pass(&self, input: TimeState, ctx: PassContext) -> HashMap<PinId, TimeUpdate> {
        let duration_1 = ctx.duration_back(Self::INPUT_1);
        let Some(duration_1) = duration_1 else {
            // First input is infinite, forward time update without change
            return HashMap::from([
                (Self::INPUT_1.into(), input.update),
                (Self::INPUT_2.into(), TimeUpdate::Delta(0.)),
            ]);
        };

        let prev_time_state = ctx.prev_time_fwd_opt().unwrap_or(input);
        let prev_time = prev_time_state.time;

        if input.time < duration_1 {
            // Current frame ends in first clip
            HashMap::from([
                (Self::INPUT_1.into(), input.update),
                (Self::INPUT_2.into(), TimeUpdate::Absolute(0.)),
            ])
        } else {
            // Current frame ends in second clip
            // Sometimes a frame can start in the first clip and end in the second.
            // In such cases, the given update delta will encompass the period spent in the first
            // frame, which will desync the clip.
            // subtracting the extraneous dt will counter that.
            let extraneous_dt = (duration_1 - prev_time).max(0.);
            HashMap::from([
                (Self::INPUT_1.into(), TimeUpdate::Absolute(0.)),
                (
                    Self::INPUT_2.into(),
                    match input.update {
                        TimeUpdate::Absolute(t) => TimeUpdate::Absolute(t - duration_1),
                        TimeUpdate::Delta(dt) => TimeUpdate::Delta(dt - extraneous_dt),
                    },
                ),
            ])
        }
    }

    fn time_dependent_pass(
        &self,
        mut inputs: HashMap<PinId, PoseFrame>,
        ctx: PassContext,
    ) -> Option<PoseFrame> {
        let in_pose_1 = inputs.remove(Self::INPUT_1).unwrap();
        let in_pose_2 = inputs.remove(Self::INPUT_2).unwrap();
        let time = ctx.time_fwd().time;

        let duration_1 = ctx.duration_back(Self::INPUT_1);
        let duration_2 = ctx.duration_back(Self::INPUT_2);

        let out_pose;

        if let Some(duration_1) = duration_1 {
            out_pose =
                in_pose_1.chain(&in_pose_2, duration_1, duration_2.unwrap_or(f32::MAX), time);
        } else {
            out_pose = in_pose_1;
        }

        Some(out_pose)
    }

    fn parameter_input_spec(&self, _: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self, _: SpecContext) -> HashMap<PinId, ParamSpec> {
        HashMap::new()
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashSet<PinId> {
        HashSet::from([Self::INPUT_1.into(), Self::INPUT_2.into()])
    }

    fn pose_output_spec(&self, _: SpecContext) -> bool {
        true
    }

    fn display_name(&self) -> String {
        " Chain".into()
    }
}
