use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::errors::GraphError;
use crate::core::pose::Pose;
use crate::core::prelude::DataSpec;
use crate::prelude::{PassContext, SpecContext};
use crate::utils::unwrap::UnwrapVal;
use bevy::reflect::std_traits::ReflectDefault;
use bevy::reflect::Reflect;

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
pub struct SpeedNode;

impl SpeedNode {
    pub const IN_POSE: &'static str = "pose";
    pub const IN_TIME: &'static str = "time";
    pub const OUT_POSE: &'static str = "pose";
    pub const SPEED: &'static str = "speed";

    pub fn new() -> Self {
        Self
    }
}

impl NodeLike for SpeedNode {
    fn duration(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let speed = ctx.data_back(Self::SPEED)?.unwrap_f32();
        let out_duration = if speed == 0. {
            None
        } else {
            let duration = ctx.duration_back(Self::IN_TIME)?;
            duration.as_ref().map(|duration| duration / speed)
        };
        ctx.set_duration_fwd(out_duration);
        Ok(())
    }

    fn update(&self, mut ctx: PassContext) -> Result<(), GraphError> {
        let speed = ctx.data_back(Self::SPEED)?.unwrap_f32();
        let input = ctx.time_update_fwd()?;
        let fw_upd = match input {
            TimeUpdate::Delta(dt) => TimeUpdate::Delta(dt * speed),
            TimeUpdate::Absolute(t) => TimeUpdate::Absolute(t),
        };

        ctx.set_time_update_back(Self::IN_TIME, fw_upd);
        let mut in_pose: Pose = ctx.data_back(Self::IN_POSE)?.val();

        if speed != 0. {
            in_pose.timestamp /= speed.abs();
        }

        ctx.set_time(in_pose.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, in_pose);

        Ok(())
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        [(Self::IN_TIME.into(), ())].into()
    }

    fn time_output_spec(&self, _ctx: SpecContext) -> Option<()> {
        Some(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [
            (Self::SPEED.into(), DataSpec::F32),
            (Self::IN_POSE.into(), DataSpec::Pose),
        ]
        .into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose)].into()
    }

    fn display_name(&self) -> String {
        "⌚ Speed".into()
    }
}
