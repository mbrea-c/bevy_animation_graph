pub mod saving;

use bevy::{
    ecs::{
        system::{In, ResMut},
        world::World,
    },
    log::error,
};
use egui_dock::DockState;
use saving::{handle_save_action, SaveAction};

use super::{
    core::{ViewAction, ViewState},
    windows::WindowAction,
    UiState,
};

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
    Window(WindowAction),
    View(ViewAction),
    Save(SaveAction),
}

pub fn handle_editor_action_queue(world: &mut World, queue: PushQueue<EditorAction>) {
    for action in queue.0 {
        handle_editor_action(world, action);
    }
}

pub fn handle_editor_action(world: &mut World, action: EditorAction) {
    match action {
        EditorAction::Window(window_action) => {
            if let Err(err) = world.run_system_cached_with(handle_window_action, window_action) {
                error!("Failed to apply window action: {:?}", err);
            }
        }
        EditorAction::View(view_action) => {
            if let Err(err) = world.run_system_cached_with(handle_view_action, view_action) {
                error!("Failed to apply view action: {:?}", err);
            }
        }
        EditorAction::Save(save_action) => handle_save_action(world, save_action),
    }
}

fn handle_view_action(In(view_action): In<ViewAction>, mut ui_state: ResMut<UiState>) {
    match view_action {
        ViewAction::Close(index) => {
            ui_state.views.remove(index);
            if let Some(idx) = ui_state.active_view {
                if idx == index {
                    ui_state.active_view = None;
                } else if idx > index {
                    ui_state.active_view = Some(idx - 1);
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

fn handle_window_action(In(window_action): In<WindowAction>, mut ui_state: ResMut<UiState>) {
    ui_state.windows.handle_action(window_action);
}
