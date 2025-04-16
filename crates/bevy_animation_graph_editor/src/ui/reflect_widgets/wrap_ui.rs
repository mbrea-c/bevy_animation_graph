//! A wrapper over "InspectorUi" that allows to do mutable editing with immutable references
//! by taking advantage of buffered values.

use bevy::{
    ecs::{
        reflect::AppTypeRegistry,
        world::{CommandQueue, World},
    },
    reflect::PartialReflect,
};
use bevy_inspector_egui::reflect_inspector::{Context, InspectorUi};
use egui_dock::egui;

use super::{get_buffered, EguiInspectorBuffers, TopLevelBuffer, WidgetHash};

pub struct WrapUi<'a, 'c> {
    inspector_ui: InspectorUi<'a, 'c>,
}

impl WrapUi<'_, '_> {
    pub fn mutable_buffered<T, O>(
        &mut self,
        value: &T,
        ui: &mut egui::Ui,
        id: egui::Id,
        options: &O,
    ) -> Option<T>
    where
        T: PartialReflect + Clone + WidgetHash,
        O: 'static,
    {
        self.initialize_buffer_if_missing::<T>();
        // First we get the buffered value
        let buf_val = get_buffered::<T, T, TopLevelBuffer>(
            self.inspector_ui.context.world.as_mut().unwrap(),
            value,
            id,
        );

        let is_changed = self
            .inspector_ui
            .ui_for_reflect_with_options(buf_val, ui, id, options);

        is_changed.then(|| buf_val.clone())
    }

    fn initialize_buffer_if_missing<T>(&mut self)
    where
        T: PartialReflect + Clone + WidgetHash,
    {
        let Some(world) = self.inspector_ui.context.world.as_mut() else {
            return;
        };

        if world
            .get_resource_mut::<EguiInspectorBuffers<T, T, TopLevelBuffer>>()
            .is_err()
        {
            unsafe {
                world
                    .world()
                    .world_mut()
                    .insert_resource(EguiInspectorBuffers::<T, T, TopLevelBuffer>::default());
            }
        }
    }
}

pub fn using_wrap_ui<T>(world: &mut World, expr: impl FnOnce(WrapUi) -> T) -> T {
    let unsafe_world = world.as_unsafe_world_cell();
    let type_registry = unsafe {
        unsafe_world
            .get_resource::<AppTypeRegistry>()
            .unwrap()
            .0
            .clone()
    };
    let type_registry = type_registry.read();
    let mut queue = CommandQueue::default();
    let mut cx = Context {
        world: Some(unsafe { unsafe_world.world_mut() }.into()),
        queue: Some(&mut queue),
    };

    let env = InspectorUi::for_bevy(&type_registry, &mut cx);
    let wrap_ui = WrapUi { inspector_ui: env };

    expr(wrap_ui)
}
