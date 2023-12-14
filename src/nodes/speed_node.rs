use crate::core::animation_graph::{
    OptParamSpec, ParamSpec, ParamValue, PinId, TimeState, TimeUpdate,
};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::frame::PoseFrame;
use crate::prelude::{DurationData, PassContext, SpecContext};
use bevy::utils::HashSet;
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
        _: HashMap<PinId, ParamValue>,
        _: PassContext,
    ) -> HashMap<PinId, ParamValue> {
        HashMap::new()
    }

    fn duration_pass(
        &self,
        inputs: HashMap<PinId, Option<f32>>,
        ctx: PassContext,
    ) -> Option<DurationData> {
        let speed = ctx.parameter_back(Self::SPEED).unwrap_f32();

        let out_duration = if speed == 0. {
            None
        } else {
            let duration = inputs.get(Self::INPUT).unwrap();
            duration.as_ref().map(|duration| duration / speed)
        };

        Some(out_duration)
    }

    fn time_pass(&self, input: TimeState, ctx: PassContext) -> HashMap<PinId, TimeUpdate> {
        let speed = ctx.parameter_back(Self::SPEED).unwrap_f32();
        let fw_upd = match input.update {
            TimeUpdate::Delta(dt) => TimeUpdate::Delta(dt * speed),
            TimeUpdate::Absolute(t) => TimeUpdate::Absolute(t * speed),
        };
        HashMap::from([(Self::INPUT.into(), fw_upd)])
    }

    fn time_dependent_pass(
        &self,
        mut inputs: HashMap<PinId, PoseFrame>,
        ctx: PassContext,
    ) -> Option<PoseFrame> {
        let mut in_pose_frame = inputs.remove(Self::INPUT).unwrap();
        let speed = ctx.parameter_back(Self::SPEED).unwrap_f32();

        if speed != 0. {
            in_pose_frame.map_ts(|t| t / speed);
        }

        Some(in_pose_frame)
    }

    fn parameter_input_spec(&self, _: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::from([(Self::SPEED.into(), ParamSpec::F32.into())])
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
        "󰓅 Speed".into()
    }
}
