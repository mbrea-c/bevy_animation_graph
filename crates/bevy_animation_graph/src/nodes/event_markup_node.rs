use crate::core::animation_graph::{PinMap, TimeUpdate};
use crate::core::animation_node::{NodeLike, ReflectNodeLike};
use crate::core::context::SpecContext;
use crate::core::context::new_context::NodeContext;
use crate::core::edge_data::{DataSpec, EventQueue};
use crate::core::errors::GraphError;
use crate::core::event_track::EventTrack;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// This node enables "decorating" arbitrary animations with event tracks.
///
/// The timestamp of upstream animation data will be used for sampling the event tracks.
/// If a "percentage through event" time update is received from downstream, the event tracks
/// are used to convert it to an absolute timestamp and forward it to upstream nodes.
///
/// Useful when you synthesize some animations using the graph, but still want some event markup to
/// apply for synchronization and/or gameplay effects.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Default, NodeLike, Serialize, Deserialize)]
#[type_path = "bevy_animation_graph::builtin_nodes"]
pub struct EventMarkupNode {
    pub event_tracks: HashMap<String, EventTrack>,
}

impl Default for EventMarkupNode {
    fn default() -> Self {
        Self::new(HashMap::default())
    }
}

impl EventMarkupNode {
    pub const IN_POSE: &'static str = "pose";
    pub const IN_EVENT_QUEUE: &'static str = "events";
    pub const IN_TIME: &'static str = "time";
    pub const OUT_POSE: &'static str = "pose";
    pub const OUT_EVENT_QUEUE: &'static str = "events";

    pub fn new(event_tracks: HashMap<String, EventTrack>) -> Self {
        Self { event_tracks }
    }
}

impl NodeLike for EventMarkupNode {
    fn duration(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let duration = ctx.duration_back(Self::IN_TIME)?;
        ctx.set_duration_fwd(duration);
        Ok(())
    }

    fn update(&self, mut ctx: NodeContext) -> Result<(), GraphError> {
        let input = ctx.time_update_fwd()?;

        let processed_update = match &input {
            dt @ TimeUpdate::Delta(_) => dt.clone(),
            at @ TimeUpdate::Absolute(_) => at.clone(),
            pt @ TimeUpdate::PercentOfEvent {
                percent,
                event,
                track,
            } => {
                if let Some(abs_time) = self
                    .event_tracks
                    .get(track)
                    .and_then(|track| track.seek_event(event, *percent))
                {
                    TimeUpdate::Absolute(abs_time)
                } else {
                    pt.clone()
                }
            }
        };

        ctx.set_time_update_back(Self::IN_TIME, processed_update);

        let in_events = ctx.data_back(Self::IN_EVENT_QUEUE)?.into_event_queue()?;
        let in_pose = ctx.data_back(Self::IN_POSE)?.into_pose()?;

        let sampled_events = EventQueue {
            events: self
                .event_tracks
                .values()
                .flat_map(|track| track.sample(in_pose.timestamp))
                .collect(),
        };

        ctx.set_time(in_pose.timestamp);

        ctx.set_data_fwd(Self::OUT_POSE, in_pose);
        ctx.set_data_fwd(Self::OUT_EVENT_QUEUE, in_events.concat(sampled_events));

        Ok(())
    }

    fn data_input_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::IN_POSE.into(), DataSpec::Pose)].into()
    }

    fn data_output_spec(&self, _: SpecContext) -> PinMap<DataSpec> {
        [(Self::OUT_POSE.into(), DataSpec::Pose)].into()
    }

    fn time_input_spec(&self, _: SpecContext) -> PinMap<()> {
        [(Self::IN_TIME.into(), ())].into()
    }

    fn time_output_spec(&self, _: SpecContext) -> Option<()> {
        Some(())
    }

    fn display_name(&self) -> String {
        "Event Markup".into()
    }
}
