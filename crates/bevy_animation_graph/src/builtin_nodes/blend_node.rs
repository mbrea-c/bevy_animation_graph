use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::DataSpec;
use crate::core::errors::GraphError;
use crate::core::pose::Pose;
use crate::interpolation::linear::InterpolateLinear;
use bevy::prelude::*;
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
}

impl BlendNode {
    pub const FACTOR: &'static str = "factor";
    pub const IN_POSE_A: &'static str = "pose_a";
    pub const IN_TIME_A: &'static str = "time_a";
    pub const IN_EVENT_A: &'static str = "events_a";
    pub const IN_POSE_B: &'static str = "pose_b";
    pub const IN_TIME_B: &'static str = "time_b";
    pub const IN_EVENT_B: &'static str = "events_b";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(mode: BlendMode, sync_mode: BlendSyncMode) -> Self {
        Self { mode, sync_mode }
    }
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

        let in_frame_2: Pose = ctx.data_back(Self::IN_POSE_B)?.into_pose()?;

        let out = match self.mode {
            BlendMode::LinearInterpolate => {
                let alpha = ctx.data_back(Self::FACTOR)?.as_f32()?;
                in_frame_1.interpolate_linear(&in_frame_2, alpha)
            }
            BlendMode::Additive => {
                let alpha = ctx.data_back(Self::FACTOR)?.as_f32()?;
                in_frame_1.additive_blend(&in_frame_2, alpha)
            }
            BlendMode::Difference => in_frame_1.difference(&in_frame_2),
        };

        ctx.set_time(out.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, out);

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        let mut input_data = vec![
            (Self::FACTOR.into(), DataSpec::F32),
            (Self::IN_POSE_A.into(), DataSpec::Pose),
            (Self::IN_POSE_B.into(), DataSpec::Pose),
        ];

        if matches!(self.sync_mode, BlendSyncMode::EventTrack(_)) {
            input_data.push((Self::IN_EVENT_A.into(), DataSpec::EventQueue));
            input_data.push((Self::IN_EVENT_B.into(), DataSpec::EventQueue));
        }

        input_data.into_iter().collect()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        [(Self::IN_TIME_A.into(), ()), (Self::IN_TIME_B.into(), ())].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "∑ Blend".into()
    }
}
