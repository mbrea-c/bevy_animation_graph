use bevy::{platform::collections::HashMap, prelude::*};
use bevy_animation_graph_core::{
    animation_graph::TimeUpdate,
    animation_node::{NodeLike, ReflectNodeLike},
    context::{new_context::NodeContext, spec_context::SpecContext},
    edge_data::{DataSpec, events::EventQueue},
    errors::GraphError,
    event_track::EventTrack,
};
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

    fn spec(&self, mut ctx: SpecContext) -> Result<(), GraphError> {
        ctx //
            .add_input_data(Self::IN_POSE, DataSpec::Pose)
            .add_input_time(Self::IN_TIME);
        ctx //
            .add_output_data(Self::OUT_POSE, DataSpec::Pose)
            .add_output_data(Self::OUT_EVENT_QUEUE, DataSpec::EventQueue)
            .add_output_time();

        Ok(())
    }

    fn display_name(&self) -> String {
        "Event Markup".into()
    }
}
