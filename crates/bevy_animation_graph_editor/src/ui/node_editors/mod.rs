use std::any::Any;

use bevy::{app::App, ecs::world::World, reflect::CreateTypeData};
use bevy_animation_graph::{
    builtin_nodes::{
        blend_space_node::BlendSpaceNode, constants::Constants, flip_lr_node::FlipLRNode,
        global_input::GlobalInput, ragdoll::const_ragdoll_config::ConstRagdollConfig,
    },
    core::animation_node::NodeLike,
};

use crate::ui::node_editors::new_reflect_editor::NewReflectNodeEditor;

pub mod blend_space;
pub mod flip_lr;
pub mod new_reflect_editor;
pub mod proxy_reflect_editor;
pub mod ragdoll_config;
pub mod reflect_editor;

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

impl<T> CreateTypeData<T> for ReflectEditable
where
    T: Editable,
{
    fn create_type_data(_: ()) -> Self {
        Self {
            get_editor: reflect_get_editor::<T>,
        }
    }
}

fn reflect_get_editor<T: Editable>(value: &dyn Any) -> Box<dyn DynNodeEditor> {
    let static_value = value.downcast_ref::<T>().expect("Reflection type mismatch");
    Box::new(static_value.get_editor())
}

impl Editable for GlobalInput {
    type Editor = NewReflectNodeEditor;

    fn get_editor(&self) -> Self::Editor {
        NewReflectNodeEditor
    }
}

impl Editable for Constants {
    type Editor = NewReflectNodeEditor;

    fn get_editor(&self) -> Self::Editor {
        NewReflectNodeEditor
    }
}

pub fn register_node_editables(app: &mut App) {
    app.register_type_data::<ConstRagdollConfig, ReflectEditable>();
    app.register_type_data::<BlendSpaceNode, ReflectEditable>();
    app.register_type_data::<GlobalInput, ReflectEditable>();
    app.register_type_data::<Constants, ReflectEditable>();
    app.register_type_data::<FlipLRNode, ReflectEditable>();
}
