use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_graph::TimeUpdate,
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::{DataSpec, DataValue, bone_mask::BoneMask},
    errors::GraphError,
    interpolation::{
        additive::AdditiveInterpolator, difference::DifferenceInterpolator,
        linear::LinearInterpolator,
    },
    pose::Pose,
};
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[reflect(Default, Serialize)]
pub enum BlendMode {
    #[default]
    LinearInterpolate,
    Additive,
    Difference,
}

#[derive(Reflect, Clone, Debug, Default, Serialize, Deserialize)]
#[reflect(Default)]
pub enum BlendSyncMode {
    /// Sets the absolute timestamp of input 2 equal to the timestamp from input 1
    #[default]
    Absolute,
    /// Propagates the same time update that was received, does not try to sync the inputs.
    NoSync,
    /// Synchronizes
    EventTrack(String),
}

#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct BlendNode {
    pub mode: BlendMode,
    pub sync_mode: BlendSyncMode,
    pub use_bone_mask: bool,
}

impl BlendNode {
    pub const FACTOR: &'static str = "factor";
    pub const IN_POSE_A: &'static str = "pose_a";
    pub const IN_TIME_A: &'static str = "time_a";
    pub const IN_EVENT_A: &'static str = "events_a";
    pub const IN_POSE_B: &'static str = "pose_b";
    pub const IN_TIME_B: &'static str = "time_b";
    pub const IN_EVENT_B: &'static str = "events_b";
    pub const IN_BONE_MASK: &'static str = "bone_mask";
    pub const OUT_POSE: &'static str = "pose";
}

impl NodeLike for BlendNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let duration_1 = ctx.duration_back(Self::IN_TIME_A)?;
        let duration_2 = ctx.duration_back(Self::IN_TIME_B)?;

        let out_duration = match (duration_1, duration_2) {
            (Some(duration_1), Some(duration_2)) => Some(duration_1.max(duration_2)),
            (Some(duration_1), None) => Some(duration_1),
            (None, Some(duration_2)) => Some(duration_2),
            (None, None) => None,
        };

        ctx.set_duration_fwd(out_duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;

        ctx.set_time_update_back(Self::IN_TIME_A, input.clone());
        let in_frame_1: Pose = ctx.data_back(Self::IN_POSE_A)?.into_pose()?;

        match &self.sync_mode {
            BlendSyncMode::Absolute => {
                ctx.set_time_update_back(
                    Self::IN_TIME_B,
                    TimeUpdate::Absolute(in_frame_1.timestamp),
                );
            }
            BlendSyncMode::NoSync => {
                ctx.set_time_update_back(Self::IN_TIME_B, input);
            }
            BlendSyncMode::EventTrack(track_name) => {
                let event_queue_1 = ctx.data_back(Self::IN_EVENT_A)?.into_event_queue()?;
                if let Some(event) = event_queue_1.events.iter().find(|ev| {
                    ev.track
                        .as_ref()
                        .map(|track| track == track_name)
                        .unwrap_or(false)
                }) {
                    ctx.set_time_update_back(
                        Self::IN_TIME_B,
                        TimeUpdate::PercentOfEvent {
                            percent: event.percentage,
                            event: event.event.clone(),
                            track: track_name.clone(),
                        },
                    );
                } else {
                    ctx.set_time_update_back(Self::IN_TIME_B, input);
                }
            }
        };

        let in_frame_2 = ctx.data_back(Self::IN_POSE_B)?.into_pose()?;
        let bone_mask = ctx
            .data_back(Self::IN_BONE_MASK)
            .unwrap_or_else(|_| DataValue::BoneMask(BoneMask::all()))
            .into_bone_mask()?;

        let mut base = in_frame_1;
        let overlay = in_frame_2;

        match self.mode {
            BlendMode::LinearInterpolate => {
                let interpolator = LinearInterpolator { bone_mask };
                let alpha = ctx.data_back(Self::FACTOR)?.as_f32()?;
                interpolator.interpolate_pose(&mut base, &overlay, alpha);
            }
            BlendMode::Additive => {
                let interpolator = AdditiveInterpolator { bone_mask };
                let alpha = ctx.data_back(Self::FACTOR)?.as_f32()?;
                interpolator.interpolate_pose(&mut base, &overlay, alpha);
            }
            BlendMode::Difference => {
                let interpolator = DifferenceInterpolator { bone_mask };
                interpolator.interpolate_pose(&mut base, &overlay);
            }
        };

        ctx.set_time(base.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, base);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        // Input
        ctx.add_input_data(Self::FACTOR, DataSpec::F32);

        if self.use_bone_mask {
            ctx.add_input_data(Self::IN_BONE_MASK, DataSpec::BoneMask);
        }

        ctx.add_input_data(Self::IN_POSE_A, DataSpec::Pose);
        if matches!(&self.sync_mode, BlendSyncMode::EventTrack(_)) {
            ctx.add_input_data(Self::IN_EVENT_A, DataSpec::EventQueue);
        }
        ctx.add_input_time(Self::IN_TIME_A);

        ctx.add_input_data(Self::IN_POSE_B, DataSpec::Pose);
        if matches!(&self.sync_mode, BlendSyncMode::EventTrack(_)) {
            ctx.add_input_data(Self::IN_EVENT_B, DataSpec::EventQueue);
        }
        ctx.add_input_time(Self::IN_TIME_B);

        // Output
        ctx.add_output_data(Self::OUT_POSE, DataSpec::Pose)
            .add_output_time();

        Ok(())
    }

    fn display_name(&self) -> String {
        "∑ Blend".into()
    }
}
