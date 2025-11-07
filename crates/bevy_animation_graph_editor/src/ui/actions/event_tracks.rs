use std::fmt::Display;

use bevy::{
    asset::Assets,
    ecs::{
        system::{In, ResMut},
        world::World,
    },
    log::error,
    platform::collections::HashMap,
};
use bevy_animation_graph::{
    core::event_track::{EventTrack, TrackItem, TrackItemValue},
    nodes::EventMarkupNode,
    prelude::{AnimationGraph, GraphClip},
};
use uuid::Uuid;

use crate::ui::native_windows::event_track_editor::TargetTracks;

use super::saving::DirtyAssets;

pub enum EventTrackAction {
    NewEvent(NewEventAction),
    NewTrack(NewTrackAction),
    EditEvent(EditEventAction),
}

pub struct NewEventAction {
    pub target_tracks: TargetTracks,
    pub track_id: String,
    pub item: TrackItem,
}

pub struct NewTrackAction {
    pub target_tracks: TargetTracks,
    pub track_id: String,
}

pub struct EditEventAction {
    pub target_tracks: TargetTracks,
    pub track_id: String,
    pub event_id: Uuid,
    pub item: TrackItemValue,
}

pub fn handle_event_track_action(world: &mut World, action: EventTrackAction) {
    match action {
        EventTrackAction::NewEvent(action) => {
            let _ = world
                .run_system_cached_with(handle_new_event_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        EventTrackAction::NewTrack(action) => {
            let _ = world
                .run_system_cached_with(handle_new_track_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        EventTrackAction::EditEvent(action) => {
            let _ = world
                .run_system_cached_with(handle_edit_event_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
    }
}

pub fn handle_new_event_system(
    In(action): In<NewEventAction>,
    mut graph_clip_assets: ResMut<Assets<GraphClip>>,
    mut animation_graph_assets: ResMut<Assets<AnimationGraph>>,
    mut dirty_assets: ResMut<DirtyAssets>,
) {
    edit_tracks(
        &action.target_tracks,
        &mut graph_clip_assets,
        &mut animation_graph_assets,
        &mut dirty_assets,
        |t| {
            if let Some(t) = t.get_mut(&action.track_id) {
                t.add_item_preassigned_id(action.item)
            }
        },
    );
}

pub fn handle_new_track_system(
    In(action): In<NewTrackAction>,
    mut graph_clip_assets: ResMut<Assets<GraphClip>>,
    mut animation_graph_assets: ResMut<Assets<AnimationGraph>>,
    mut dirty_assets: ResMut<DirtyAssets>,
) {
    edit_tracks(
        &action.target_tracks,
        &mut graph_clip_assets,
        &mut animation_graph_assets,
        &mut dirty_assets,
        |t| {
            if !t.contains_key(&action.track_id) {
                t.insert(
                    action.track_id.clone(),
                    EventTrack {
                        name: action.track_id,
                        events: Vec::new(),
                    },
                );
            }
        },
    );
}

pub fn handle_edit_event_system(
    In(action): In<EditEventAction>,
    mut graph_clip_assets: ResMut<Assets<GraphClip>>,
    mut animation_graph_assets: ResMut<Assets<AnimationGraph>>,
    mut dirty_assets: ResMut<DirtyAssets>,
) {
    edit_tracks(
        &action.target_tracks,
        &mut graph_clip_assets,
        &mut animation_graph_assets,
        &mut dirty_assets,
        |tracks| {
            if let Some(t) = tracks.get_mut(&action.track_id) {
                t.edit_item(action.event_id, |_| action.item)
            }
        },
    );
}

fn edit_tracks<F, T>(
    target: &TargetTracks,
    graph_clip_assets: &mut Assets<GraphClip>,
    animation_graph_assets: &mut Assets<AnimationGraph>,
    dirty_assets: &mut DirtyAssets,
    f: F,
) -> Option<T>
where
    F: FnOnce(&mut HashMap<String, EventTrack>) -> T,
{
    match target {
        TargetTracks::Clip(handle) => {
            dirty_assets.add(handle.clone());
            graph_clip_assets
                .get_mut(handle.id())
                .map(|c| c.event_tracks_mut())
                .map(f)
        }
        TargetTracks::GraphNode { graph, node } => {
            dirty_assets.add(graph.clone());
            animation_graph_assets
                .get_mut(graph.id())
                .and_then(|g| g.nodes.get_mut(node))
                .and_then(|n| n.inner.as_any_mut().downcast_mut::<EventMarkupNode>())
                .map(|n| &mut n.event_tracks)
                .map(f)
        }
    }
}

fn handle_system_error<Err: Display>(err: Err) {
    error!("Failed to apply event track action: {}", err);
}
