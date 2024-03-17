use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{AnimationNode, AnimationNodeType, NodeLike};
use crate::core::duration_data::DurationData;
use crate::core::errors::GraphError;
use crate::core::pose::{Pose, PoseSpec};
use crate::prelude::{OptParamSpec, ParamSpec, PassContext, SpecContext};
use bevy::reflect::std_traits::ReflectDefault;
use bevy::reflect::Reflect;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default)]
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
    fn duration_pass(&self, mut ctx: PassContext) -> Result<Option<DurationData>, GraphError> {
        let speed = ctx.parameter_back(Self::SPEED)?.unwrap_f32();

        let out_duration = if speed == 0. {
            None
        } else {
            let duration = ctx.duration_back(Self::INPUT)?;
            duration.as_ref().map(|duration| duration / speed)
        };

        Ok(Some(out_duration))
    }

    fn pose_pass(
        &self,
        input: TimeUpdate,
        mut ctx: PassContext,
    ) -> Result<Option<Pose>, GraphError> {
        let speed = ctx.parameter_back(Self::SPEED)?.unwrap_f32();
        let fw_upd = match input {
            TimeUpdate::Delta(dt) => TimeUpdate::Delta(dt * speed),
            TimeUpdate::Absolute(t) => TimeUpdate::Absolute(t * speed),
        };
        let mut in_pose = ctx.pose_back(Self::INPUT, fw_upd)?;

        if speed != 0. {
            in_pose.timestamp /= speed.abs();
        }

        Ok(Some(in_pose))
    }

    fn parameter_input_spec(&self, _: SpecContext) -> PinMap<OptParamSpec> {
        [(Self::SPEED.into(), ParamSpec::F32.into())].into()
    }

    fn pose_input_spec(&self, _: SpecContext) -> PinMap<PoseSpec> {
        [(Self::INPUT.into(), PoseSpec::Any)].into()
    }

    fn pose_output_spec(&self, _: SpecContext) -> Option<PoseSpec> {
        Some(PoseSpec::Any)
    }

    fn display_name(&self) -> String {
        "⌚ Speed".into()
    }
}
