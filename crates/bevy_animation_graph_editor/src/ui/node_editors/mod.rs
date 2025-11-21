pub mod ragdoll_config;
pub mod reflect_editor;

use std::any::Any;

use bevy::{app::App, ecs::world::World, reflect::FromType};
use bevy_animation_graph::{
    builtin_nodes::const_ragdoll_config::ConstRagdollConfig, prelude::NodeLike,
};

pub trait NodeEditor: 'static {
    type Target: 'static;
    fn show(&self, ui: &mut egui::Ui, world: &mut World, node: &mut Self::Target)
    -> egui::Response;
}

pub trait DynNodeEditor: 'static {
    fn show_dyn(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        node: &mut dyn NodeLike,
    ) -> egui::Response;
}

impl<E> DynNodeEditor for E
where
    E: NodeEditor,
{
    fn show_dyn(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        node: &mut dyn NodeLike,
    ) -> egui::Response {
        self.show(
            ui,
            world,
            node.as_any_mut()
                .downcast_mut()
                .expect("Mismatched node editor used"),
        )
    }
}

pub trait Editable: 'static {
    type Editor: DynNodeEditor;

    fn get_editor(&self) -> Self::Editor;
}

#[derive(Clone)]
pub struct ReflectEditable {
    // If needed, we can also store the type ID of the editor in this struct
    pub get_editor: fn(&dyn Any) -> Box<dyn DynNodeEditor>,
}

impl<T> FromType<T> for ReflectEditable
where
    T: Editable,
{
    fn from_type() -> Self {
        Self {
            get_editor: reflect_get_editor::<T>,
        }
    }
}

fn reflect_get_editor<T: Editable>(value: &dyn Any) -> Box<dyn DynNodeEditor> {
    let static_value = value.downcast_ref::<T>().expect("Reflection type mismatch");
    Box::new(static_value.get_editor())
}

pub fn register_node_editables(app: &mut App) {
    app.register_type_data::<ConstRagdollConfig, ReflectEditable>();
}
