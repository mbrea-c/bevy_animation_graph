use bevy::{ecs::world::World, platform::collections::HashMap};
use egui_dock::egui;
use uuid::Uuid;

use super::{
    actions::window::{DynWindowAction, WindowAction},
    core::LegacyEditorWindowContext,
};

pub trait EditorWindowExtension: std::fmt::Debug + Send + Sync + 'static {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut LegacyEditorWindowContext);
    fn display_name(&self) -> String;
    #[allow(unused_variables)]
    fn handle_action(&mut self, action: DynWindowAction) {}
    fn closeable(&self) -> bool {
        false
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct EditorWindow {
    id: WindowId,
    window: Box<dyn EditorWindowExtension>,
}

impl EditorWindowExtension for EditorWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, ctx: &mut LegacyEditorWindowContext) {
        self.window.ui(ui, world, ctx);
    }

    fn display_name(&self) -> String {
        self.window.display_name()
    }

    fn handle_action(&mut self, event: DynWindowAction) {
        self.window.handle_action(event);
    }

    fn closeable(&self) -> bool {
        self.window.closeable()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(Uuid);

#[derive(Default)]
pub struct Windows {
    windows: HashMap<WindowId, EditorWindow>,
}

impl Windows {
    pub fn open(&mut self, window: impl EditorWindowExtension) -> WindowId {
        let id = WindowId(Uuid::new_v4());
        self.windows.insert(
            id,
            EditorWindow {
                id,
                window: Box::new(window),
            },
        );

        id
    }

    pub fn close(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }

    pub fn window_exists(&self, window_id: WindowId) -> bool {
        self.windows.contains_key(&window_id)
    }

    pub fn get_window(&self, id: WindowId) -> Option<&EditorWindow> {
        self.windows.get(&id)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut EditorWindow> {
        self.windows.get_mut(&id)
    }

    pub fn name_for_window(&self, id: WindowId) -> String {
        self.get_window(id)
            .map(|w| w.display_name())
            .unwrap_or_else(|| "<WINDOW ERROR>".into())
    }

    pub fn handle_action(&mut self, event: WindowAction) {
        if let Some(w) = self.get_window_mut(event.target) {
            w.handle_action(event.action);
        }
    }

    pub fn with_window_mut<F, T>(&mut self, window_id: WindowId, f: F) -> Option<T>
    where
        F: FnOnce(&mut EditorWindow, &mut Self) -> T,
    {
        let mut window = self.windows.remove(&window_id)?;
        let result = f(&mut window, self);
        self.windows.insert(window_id, window);

        Some(result)
    }
}
