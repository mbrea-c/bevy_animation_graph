use crate::chaining::Chainable;
use crate::core::animation_graph::{
    EdgePath, EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::graph_context::{GraphContext, GraphContextTmp};
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, Clone, Debug)]
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
        _inputs: HashMap<NodeInput, EdgeValue>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<NodeInput, Option<f32>>,
        _name: &str,
        _path: &EdgePath,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> Option<f32> {
        let source_duration_1 = *inputs.get(Self::INPUT_1.into()).unwrap();
        let source_duration_2 = *inputs.get(Self::INPUT_2.into()).unwrap();

        match (source_duration_1, source_duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1 + duration_2),
            (Some(_), None) => None,
            (None, Some(_)) => None,
            (None, None) => None,
        }
    }

    fn time_pass(
        &self,
        input: TimeState,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, TimeUpdate> {
        let durations = context.get_durations(name).unwrap();
        let duration_1 = durations.upstream.get(Self::INPUT_1).unwrap();
        let Some(duration_1) = duration_1 else {
            // First input is infinite, forward time update without change
            return HashMap::from([
                (Self::INPUT_1.into(), input.update),
                (Self::INPUT_2.into(), TimeUpdate::Delta(0.)),
            ]);
        };

        let prev_time = context
            .get_other_times(name, path)
            .map(|c| c.downstream.time)
            .unwrap_or(input.time);

        if input.time < *duration_1 {
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
            let extraneous_dt = (*duration_1 - prev_time).max(0.);
            HashMap::from([
                (Self::INPUT_1.into(), TimeUpdate::Absolute(0.)),
                (
                    Self::INPUT_2.into(),
                    match input.update {
                        TimeUpdate::Absolute(t) => TimeUpdate::Absolute(t - *duration_1),
                        TimeUpdate::Delta(dt) => TimeUpdate::Delta(dt - extraneous_dt),
                    },
                ),
            ])
        }
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        name: &str,
        path: &EdgePath,
        context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let in_pose_1 = inputs
            .get(Self::INPUT_1.into())
            .unwrap()
            .clone()
            .unwrap_pose_frame();
        let in_pose_2 = inputs
            .get(Self::INPUT_2.into())
            .unwrap()
            .clone()
            .unwrap_pose_frame();
        let time = context.get_times(name, path).unwrap();
        let durations = context.get_durations(name).unwrap();

        let time = time.downstream.time;

        let duration_1 = *durations.upstream.get(Self::INPUT_1).unwrap();
        let duration_2 = *durations.upstream.get(Self::INPUT_2).unwrap();

        let out_pose;

        if let Some(duration_1) = duration_1 {
            out_pose =
                in_pose_1.chain(&in_pose_2, duration_1, duration_2.unwrap_or(f32::MAX), time);
        } else {
            out_pose = in_pose_1;
        }

        HashMap::from([(Self::OUTPUT.into(), EdgeValue::PoseFrame(out_pose))])
    }

    fn parameter_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::new()
    }

    fn parameter_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::new()
    }

    fn duration_input_spec(&self) -> HashMap<NodeInput, ()> {
        HashMap::from([(Self::INPUT_1.into(), ()), (Self::INPUT_2.into(), ())])
    }

    fn time_dependent_input_spec(&self) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([
            (Self::INPUT_1.into(), EdgeSpec::PoseFrame),
            (Self::INPUT_2.into(), EdgeSpec::PoseFrame),
        ])
    }

    fn time_dependent_output_spec(&self) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }
}
