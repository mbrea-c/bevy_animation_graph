use crate::core::animation_graph::{PinId, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::frame::{PoseFrame, PoseFrameType};
use crate::prelude::{OptParamSpec, ParamSpec, PassContext, SpecContext};
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
    fn duration_pass(&self, mut ctx: PassContext) -> Option<DurationData> {
        let speed = ctx.parameter_back(Self::SPEED).unwrap_f32();

        let out_duration = if speed == 0. {
            None
        } else {
            let duration = ctx.duration_back(Self::INPUT);
            duration.as_ref().map(|duration| duration / speed)
        };

        Some(out_duration)
    }

    fn pose_pass(&self, input: TimeUpdate, mut ctx: PassContext) -> Option<PoseFrame> {
        let speed = ctx.parameter_back(Self::SPEED).unwrap_f32();
        let fw_upd = match input {
            TimeUpdate::Delta(dt) => TimeUpdate::Delta(dt * speed),
            TimeUpdate::Absolute(t) => TimeUpdate::Absolute(t * speed),
        };
        let mut in_pose_frame = ctx.pose_back(Self::INPUT, fw_upd);

        if speed != 0. {
            in_pose_frame.map_ts(|t| t / speed.abs());
        }

        Some(in_pose_frame)
    }

    fn parameter_input_spec(&self, _: SpecContext) -> HashMap<PinId, OptParamSpec> {
        HashMap::from([(Self::SPEED.into(), ParamSpec::F32.into())])
    }

    fn pose_input_spec(&self, _: SpecContext) -> HashSet<PinId> {
        HashSet::from([Self::INPUT.into()])
    }

    fn pose_output_spec(&self, _: SpecContext) -> Option<PoseFrameType> {
        Some(PoseFrameType::BoneSpace)
    }

    fn display_name(&self) -> String {
        "󰓅 Speed".into()
    }
}
