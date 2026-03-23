use bevy::prelude::*;
use bevy_animation_graph_core::{
    animation_graph::TimeUpdate,
    animation_node::{EditProxy, NodeLike, ReflectEditProxy, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::DataSpec,
    errors::GraphError,
};
use serde::{Deserialize, Serialize};

use crate::blend_node::BlendSyncMode;

#[derive(Reflect, Clone, Debug, Default, Serialize, Deserialize)]
#[reflect(Default, Serialize, Deserialize)]
pub struct PointElement1D {
    pub id: String,
    pub value: f32,
}

/// Blends between poses arranged along a single axis. At runtime, the two nearest
/// points to the `parameter` input are found and linearly interpolated.
///
/// Points should be sorted by value for correct behavior. If the parameter falls
/// outside the range, it is clamped to the nearest endpoint.
///
/// This node is useful, for example, to blend between idle, walk, and run
/// animations based on movement speed.
#[derive(Reflect, Clone, Debug, Default)]
#[reflect(Default, NodeLike, EditProxy)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct BlendSpace1DNode {
    pub sync_mode: BlendSyncMode,
    pub points: Vec<PointElement1D>,
}

impl BlendSpace1DNode {
    pub const PARAMETER: &'static str = "parameter";
    pub const OUT_POSE: &'static str = "pose";

    pub fn new(sync_mode: BlendSyncMode, points: Vec<PointElement1D>) -> Self {
        Self { sync_mode, points }
    }

    pub fn pose_pin_id(key: &str) -> String {
        format!("pose {key}")
    }

    pub fn events_pin_id(key: &str) -> String {
        format!("events {key}")
    }

    pub fn time_pin_id(key: &str) -> String {
        format!("time {key}")
    }

    /// Finds the two surrounding points and returns (index_low, index_high, blend_factor).
    /// The blend factor is 0.0 at the low point and 1.0 at the high point.
    /// If the parameter is outside the range, clamps to the nearest endpoint.
    /// Returns `None` if there are fewer than 2 points.
    fn find_blend_pair(&self, parameter: f32) -> Option<(usize, usize, f32)> {
        let n = self.points.len();
        if n < 2 {
            return None;
        }

        // Clamp below the first point
        if parameter <= self.points[0].value {
            return Some((0, 0, 0.0));
        }

        // Clamp above the last point
        if parameter >= self.points[n - 1].value {
            return Some((n - 1, n - 1, 0.0));
        }

        // Find the segment containing the parameter
        for i in 0..n - 1 {
            let low = self.points[i].value;
            let high = self.points[i + 1].value;
            if parameter >= low && parameter <= high {
                let range = high - low;
                let factor = if range > f32::EPSILON {
                    (parameter - low) / range
                } else {
                    0.0
                };
                return Some((i, i + 1, factor));
            }
        }

        // Fallback (shouldn't reach here with sorted points)
        Some((n - 1, n - 1, 0.0))
    }
}

impl NodeLike for BlendSpace1DNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let parameter = ctx.data_back(Self::PARAMETER)?.as_f32()?;
        let Some((low, high, factor)) = self.find_blend_pair(parameter) else {
            return Ok(());
        };

        // Return the duration of the higher-weighted input
        let master_idx = if factor <= 0.5 { low } else { high };
        let master_duration =
            ctx.duration_back(Self::time_pin_id(&self.points[master_idx].id))?;

        ctx.set_duration_fwd(master_duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;
        let parameter = ctx.data_back(Self::PARAMETER)?.as_f32()?;
        let Some((low_idx, high_idx, factor)) = self.find_blend_pair(parameter) else {
            return Ok(());
        };

        let low_key = &self.points[low_idx].id;
        let high_key = &self.points[high_idx].id;

        // Evaluate the primary (higher-weighted) input first
        let (primary_key, secondary_key, primary_factor, secondary_factor) = if factor <= 0.5 {
            (low_key, high_key, 1.0 - factor, factor)
        } else {
            (high_key, low_key, factor, 1.0 - factor)
        };

        ctx.set_time_update_back(Self::time_pin_id(primary_key), input.clone());
        let primary_pose = ctx
            .data_back(Self::pose_pin_id(primary_key))?
            .into_pose()?;

        // If both indices are the same, just output the single pose
        if low_idx == high_idx {
            ctx.set_time(primary_pose.timestamp);
            ctx.set_data_fwd(Self::OUT_POSE, primary_pose);
            return Ok(());
        }

        // Set time for secondary input based on sync mode
        match &self.sync_mode {
            BlendSyncMode::Absolute => {
                ctx.set_time_update_back(
                    Self::time_pin_id(secondary_key),
                    TimeUpdate::Absolute(primary_pose.timestamp),
                );
            }
            BlendSyncMode::NoSync => {
                ctx.set_time_update_back(Self::time_pin_id(secondary_key), input);
            }
            BlendSyncMode::EventTrack(track_name) => {
                let event_queue = ctx
                    .data_back(Self::events_pin_id(primary_key))?
                    .into_event_queue()?;
                if let Some(event) = event_queue.events.iter().find(|ev| {
                    ev.track
                        .as_ref()
                        .map(|track| track == track_name)
                        .unwrap_or(false)
                }) {
                    ctx.set_time_update_back(
                        Self::time_pin_id(secondary_key),
                        TimeUpdate::PercentOfEvent {
                            percent: event.percentage,
                            event: event.event.clone(),
                            track: track_name.clone(),
                        },
                    );
                } else {
                    ctx.set_time_update_back(Self::time_pin_id(secondary_key), input);
                }
            }
        };

        let secondary_pose = ctx
            .data_back(Self::pose_pin_id(secondary_key))?
            .into_pose()?;

        let out_pose = [
            (&primary_pose, primary_factor),
            (&secondary_pose, secondary_factor),
        ]
        .into_iter()
        .map(|(pose, f)| pose.scalar_mult(f))
        .reduce(|acc, elem| acc.linear_add(&elem))
        .unwrap()
        .normalize_quat();

        ctx.set_time(primary_pose.timestamp);
        ctx.set_data_fwd(Self::OUT_POSE, out_pose);

        Ok(())
    }

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx.add_input_data(Self::PARAMETER, DataSpec::F32);

        for p in &self.points {
            ctx.add_input_data(Self::pose_pin_id(&p.id), DataSpec::Pose);
            if matches!(&self.sync_mode, BlendSyncMode::EventTrack(_)) {
                ctx.add_input_data(Self::events_pin_id(&p.id), DataSpec::EventQueue);
            }
            ctx.add_input_time(Self::time_pin_id(&p.id));
        }

        ctx.add_output_data(Self::OUT_POSE, DataSpec::Pose)
            .add_output_time();

        Ok(())
    }

    fn display_name(&self) -> String {
        "∑ Blend space 1D".into()
    }
}

#[derive(Clone, Reflect, Serialize, Deserialize)]
pub struct BlendSpace1DProxy {
    pub sync_mode: BlendSyncMode,
    pub points: Vec<PointElement1D>,
}

impl EditProxy for BlendSpace1DNode {
    type Proxy = BlendSpace1DProxy;

    fn update_from_proxy(proxy: &Self::Proxy) -> Self {
        Self {
            sync_mode: proxy.sync_mode.clone(),
            points: proxy.points.clone(),
        }
    }

    fn make_proxy(&self) -> Self::Proxy {
        Self::Proxy {
            sync_mode: self.sync_mode.clone(),
            points: self.points.clone(),
        }
    }
}
