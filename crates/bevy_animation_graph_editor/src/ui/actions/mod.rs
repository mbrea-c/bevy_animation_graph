//! Actions are the way we mutate editor state. When UI code wants to change something, they should
//! do it through an editor action.
//!
//! This will pave the way for undo/redo support later on.

pub mod clip_preview;
pub mod event_tracks;
pub mod fsm;
pub mod graph;
pub mod saving;
pub mod window;

use std::{any::Any, cmp::Ordering, fmt::Display};

use bevy::{
    ecs::{
        system::{In, IntoSystem, ResMut, Resource, SystemInput},
        world::World,
    },
    log::error,
};
use egui_dock::DockState;
use event_tracks::{handle_event_track_action, EventTrackAction};
use fsm::{handle_fsm_action, FsmAction};
use graph::{handle_graph_action, GraphAction};
use saving::{handle_save_action, SaveAction};
use window::WindowAction;

use super::{
    core::{ViewAction, ViewState},
    windows::WindowId,
    UiState,
};

#[derive(Resource, Default)]
pub struct PendingActions {
    pub actions: PushQueue<EditorAction>,
}

/// A "push-only" queue
pub struct PushQueue<T>(Vec<T>);

impl<T> Default for PushQueue<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> PushQueue<T> {
    pub fn push(&mut self, item: T) {
        self.0.push(item);
    }
}

pub enum EditorAction {
    View(ViewAction),
    Save(SaveAction),
    EventTrack(EventTrackAction),
    Graph(GraphAction),
    Fsm(FsmAction),
    Dynamic(Box<dyn DynamicAction>),
}

pub fn handle_editor_action_queue(world: &mut World, actions: impl Iterator<Item = EditorAction>) {
    for action in actions {
        handle_editor_action(world, action);
    }
}

pub fn handle_editor_action(world: &mut World, action: EditorAction) {
    match action {
        EditorAction::View(action) => {
            if let Err(err) = world.run_system_cached_with(handle_view_action, action) {
                error!("Failed to apply view action: {:?}", err);
            }
        }
        EditorAction::Save(action) => handle_save_action(world, action),
        EditorAction::EventTrack(action) => handle_event_track_action(world, action),
        EditorAction::Graph(action) => handle_graph_action(world, action),
        EditorAction::Fsm(action) => handle_fsm_action(world, action),
        EditorAction::Dynamic(action) => action.handle(world),
    }
}

fn handle_view_action(In(view_action): In<ViewAction>, mut ui_state: ResMut<UiState>) {
    match view_action {
        ViewAction::Close(index) => {
            ui_state.views.remove(index);
            if let Some(idx) = ui_state.active_view {
                match idx.cmp(&index) {
                    Ordering::Less => {}
                    Ordering::Equal => {
                        ui_state.active_view = None;
                    }
                    Ordering::Greater => {
                        ui_state.active_view = Some(idx - 1);
                    }
                }
            }
        }
        ViewAction::Select(index) => {
            ui_state.active_view = Some(index);
        }
        ViewAction::New(name) => ui_state.views.push(ViewState {
            name,
            dock_state: DockState::new(vec![]),
        }),
    }
}

pub fn process_actions_system(world: &mut World) {
    world.resource_scope::<PendingActions, ()>(|world, mut actions| {
        handle_editor_action_queue(world, actions.actions.0.drain(..));
    });
}

impl PushQueue<EditorAction> {
    pub fn window(&mut self, window: WindowId, event: impl Any + Send + Sync) {
        self.push(EditorAction::Dynamic(Box::new(WindowAction {
            target: window,
            action: Box::new(event),
        })))
    }

    pub fn dynamic(&mut self, action: impl DynamicAction) {
        self.push(EditorAction::Dynamic(Box::new(action)));
    }
}

pub fn run_handler<I, O, M, S>(
    world: &mut World,
    msg: impl Display + 'static,
) -> impl FnOnce(S, I::Inner<'_>) + '_
where
    I: SystemInput + 'static,
    O: 'static,
    S: IntoSystem<I, O, M> + 'static,
{
    move |closure, input| {
        let _ = world
            .run_system_cached_with(closure, input)
            .inspect_err(|err| error!("{}: {}", msg, err));
    }
}

pub trait DynamicAction: Send + Sync + 'static {
    fn handle(self: Box<Self>, world: &mut World);
}
