use std::any::{Any, TypeId};

use bevy::{
    ecs::{
        system::{In, ResMut},
        world::World,
    },
    log::warn,
};

use super::{DynamicAction, run_handler};
use crate::ui::{UiState, actions::ActionContext, windows::WindowId};

pub type DynWindowAction = Box<dyn Any + Send + Sync>;

/// An editor update event aimed at a particular window.
/// How they're handled is up to the window.
pub struct WindowAction {
    pub target: WindowId,
    pub action: DynWindowAction,
}

impl DynamicAction for WindowAction {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Failed to handle window action")(Self::system, *self);
    }
}

impl WindowAction {
    pub fn system(In(window_action): In<WindowAction>, mut ui_state: ResMut<UiState>) {
        ui_state.windows.handle_action(window_action);
    }
}

/// An editor update event aimed at any window satisfying the type criteria
/// How they're handled is up to the window.
pub struct TypeTargetedWindowAction {
    pub target_window_type: TypeId,
    pub action: DynWindowAction,
}

impl DynamicAction for TypeTargetedWindowAction {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Failed to handle window action")(Self::system, *self);
    }
}

impl TypeTargetedWindowAction {
    pub fn system(In(action): In<TypeTargetedWindowAction>, mut ui_state: ResMut<UiState>) {
        if let Some(window_id) = ui_state
            .windows
            .find_window_with_type_dyn(action.target_window_type)
        {
            ui_state.windows.handle_action(WindowAction {
                target: window_id,
                action: action.action,
            });
        } else {
            warn!("Type-targeted window action did not reach any windows");
        }
    }
}

pub struct CloseWindowAction {
    pub id: WindowId,
}

impl DynamicAction for CloseWindowAction {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Failed to close window")(Self::system, *self)
    }
}

impl CloseWindowAction {
    pub fn system(In(action): In<CloseWindowAction>, mut ui_state: ResMut<UiState>) {
        ui_state.windows.close(action.id);
    }
}
