use std::any::Any;

use bevy::{ecs::world::World, utils::HashMap};
use egui_dock::egui;
use uuid::Uuid;

use super::core::EditorWindowContext;

pub trait EditorWindowExtension: std::fmt::Debug + Send + Sync + 'static {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext);
    fn display_name(&self) -> String;
    #[allow(unused_variables)]
    fn handle_action(&mut self, event: WindowAction) {}
}

#[derive(Debug)]
pub struct EditorWindow {
    id: WindowId,
    window: Box<dyn EditorWindowExtension>,
}

impl EditorWindowExtension for EditorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        self.window.ui(ui, world, ctx);
    }

    fn display_name(&self) -> String {
        self.window.display_name()
    }

    fn handle_action(&mut self, event: WindowAction) {
        self.window.handle_action(event);
    }
}

/// An editor update event aimed at a particular window.
/// How they're handled is up to the window.
pub struct WindowAction {
    pub target: WindowId,
    pub event: Box<dyn Any>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(Uuid);

#[derive(Default)]
pub struct Windows {
    windows: HashMap<Uuid, EditorWindow>,
}

impl Windows {
    pub fn open(&mut self, window: impl EditorWindowExtension) -> WindowId {
        let id = Uuid::new_v4();
        self.windows.insert(
            id,
            EditorWindow {
                id: WindowId(id),
                window: Box::new(window),
            },
        );

        WindowId(id)
    }

    pub fn get_window(&self, id: WindowId) -> Option<&EditorWindow> {
        self.windows.get(&id.0)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut EditorWindow> {
        self.windows.get_mut(&id.0)
    }

    pub fn ui_for_window(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        ctx: &mut EditorWindowContext,
    ) {
        self.windows
            .get_mut(&ctx.window_id.0)
            .map(|win| win.ui(ui, world, ctx));
    }

    pub fn name_for_window(&self, id: WindowId) -> String {
        self.get_window(id)
            .map(|w| w.display_name())
            .unwrap_or_else(|| "<WINDOW ERROR>".into())
    }

    pub fn handle_action(&mut self, event: WindowAction) {
        self.get_window_mut(event.target)
            .map(|w| w.handle_action(event));
    }
}
