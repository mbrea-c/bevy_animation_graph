use crate::core::animation_graph::{
    EdgePath, EdgeSpec, EdgeValue, NodeInput, NodeOutput, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::graph_context::{GraphContext, GraphContextTmp};
use bevy::{reflect::Reflect, utils::HashMap};

#[derive(Reflect, Clone, Debug, Default)]
pub struct SpeedNode;

impl SpeedNode {
    pub const INPUT: &'static str = "Pose In";
    pub const OUTPUT: &'static str = "Pose Out";
    pub const SPEED: &'static str = "Speed";

    pub fn new() -> Self {
        Self
    }

    pub fn wrapped(self, name: impl Into<String>) -> AnimationNode {
        AnimationNode::new_from_nodetype(name.into(), AnimationNodeType::Speed(self))
    }
}

impl NodeLike for SpeedNode {
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
        name: &str,
        _path: &EdgePath,
        context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, Option<f32>> {
        let parameters = context.get_parameters(name).unwrap();
        let speed = parameters
            .upstream
            .get(Self::SPEED)
            .unwrap()
            .clone()
            .unwrap_f32();

        let out_duration = if speed == 0. {
            None
        } else {
            let duration = inputs.get(Self::INPUT).unwrap();
            duration.as_ref().map(|duration| duration / speed)
        };

        HashMap::from([(Self::OUTPUT.into(), out_duration)])
    }

    fn time_pass(
        &self,
        input: TimeState,
        name: &str,
        _path: &EdgePath,
        context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, TimeUpdate> {
        let parameters = context.get_parameters(name).unwrap();
        let speed = parameters
            .upstream
            .get(Self::SPEED)
            .unwrap()
            .clone()
            .unwrap_f32();
        let fw_upd = match input.update {
            TimeUpdate::Delta(dt) => TimeUpdate::Delta(dt * speed),
            TimeUpdate::Absolute(t) => TimeUpdate::Absolute(t * speed),
        };
        HashMap::from([(Self::INPUT.into(), fw_upd)])
    }

    fn time_dependent_pass(
        &self,
        inputs: HashMap<NodeInput, EdgeValue>,
        name: &str,
        _path: &EdgePath,
        context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeValue> {
        let mut in_pose_frame = inputs.get(Self::INPUT).unwrap().clone().unwrap_pose_frame();
        let parameters = context.get_parameters(name).unwrap();
        let speed = parameters
            .upstream
            .get(Self::SPEED)
            .unwrap()
            .clone()
            .unwrap_f32();

        if speed != 0. {
            in_pose_frame.map_ts(|t| t / speed);
        }

        HashMap::from([(Self::OUTPUT.into(), EdgeValue::PoseFrame(in_pose_frame))])
    }

    fn parameter_input_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([(Self::SPEED.into(), EdgeSpec::F32)])
    }

    fn parameter_output_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::new()
    }

    fn time_dependent_input_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeInput, EdgeSpec> {
        HashMap::from([(Self::INPUT.into(), EdgeSpec::PoseFrame)])
    }

    fn time_dependent_output_spec(
        &self,
        _context: &mut GraphContext,
        _context_tmp: &mut GraphContextTmp,
    ) -> HashMap<NodeOutput, EdgeSpec> {
        HashMap::from([(Self::OUTPUT.into(), EdgeSpec::PoseFrame)])
    }

    fn display_name(&self) -> String {
        "󰓅 Speed".into()
    }
}
