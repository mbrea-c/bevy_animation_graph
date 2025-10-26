use std::any::{Any, TypeId, type_name};

use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::Event,
    system::command::{trigger, trigger_targets},
    world::{CommandQueue, World},
};
use egui_dock::egui;
use egui_notify::Toasts;

use crate::ui::{
    actions::{EditorAction, PushQueue},
    core::Buffers,
};

pub mod inspector;
pub mod scene_picker;

pub struct EditorWindowContext<'a> {
    pub window_entity: Entity,
    pub view_entity: Entity,
    pub notifications: &'a mut Toasts,
    pub command_queue: &'a mut CommandQueue,
    pub buffers: &'a mut Buffers,

    // Legacy stuff for backwards compat
    pub editor_actions: &'a mut PushQueue<EditorAction>,
}

#[derive(Component)]
pub struct WindowState;

impl<'a> EditorWindowContext<'a> {
    pub fn trigger(&mut self, event: impl Event) {
        self.command_queue.push(trigger(event));
    }

    pub fn trigger_window(&mut self, event: impl Event) {
        self.command_queue
            .push(trigger_targets(event, self.window_entity));
    }

    pub fn trigger_view(&mut self, event: impl Event) {
        self.command_queue
            .push(trigger_targets(event, self.view_entity));
    }
}

pub struct EditorWindowRegistrationContext {
    pub window: Entity,
}

pub trait NativeEditorWindowExtension: std::fmt::Debug + Send + Sync + 'static {
    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext);

    fn display_name(&self) -> String;

    #[allow(unused_variables)]
    fn init(&self, world: &mut World, ctx: &EditorWindowRegistrationContext) {}
    #[allow(unused_variables)]
    fn register_observers(&self, world: &mut World, ctx: &EditorWindowRegistrationContext) {}

    fn closeable(&self) -> bool {
        false
    }

    fn window_type_id(&self) -> TypeId {
        self.type_id()
    }

    fn window_type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}

#[derive(Debug)]
pub struct NativeEditorWindow {
    window: Box<dyn NativeEditorWindowExtension>,
    pub entity: Entity,
}

impl NativeEditorWindowExtension for NativeEditorWindow {
    fn init(&self, world: &mut World, ctx: &EditorWindowRegistrationContext) {
        self.window.init(world, ctx)
    }

    fn ui(&self, ui: &mut egui::Ui, world: &mut World, ctx: &mut EditorWindowContext) {
        self.window.ui(ui, world, ctx);
    }

    fn display_name(&self) -> String {
        self.window.display_name()
    }

    fn register_observers(&self, world: &mut World, ctx: &EditorWindowRegistrationContext) {
        self.window.register_observers(world, ctx)
    }

    fn closeable(&self) -> bool {
        self.window.closeable()
    }

    fn window_type_id(&self) -> TypeId {
        self.window.window_type_id()
    }

    fn window_type_name(&self) -> &'static str {
        self.window.window_type_name()
    }
}

impl NativeEditorWindow {
    pub fn create<T: NativeEditorWindowExtension>(world: &mut World, ext: T) -> Self {
        let entity = world.spawn((WindowState,)).id();

        let ctx = EditorWindowRegistrationContext { window: entity };

        ext.init(world, &ctx);
        ext.register_observers(world, &ctx);

        Self {
            window: Box::new(ext),
            entity,
        }
    }
}
