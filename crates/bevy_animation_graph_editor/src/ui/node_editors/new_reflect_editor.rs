use bevy::ecs::world::World;
use bevy_animation_graph::core::animation_node::NodeLike;

use crate::ui::{node_editors::DynNodeEditor, reflect_lib::ReflectWidgetContext};

#[derive(Default)]
pub struct NewReflectNodeEditor;

impl DynNodeEditor for NewReflectNodeEditor {
    fn show_dyn(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        node: &mut dyn NodeLike,
    ) -> egui::Response {
        ReflectWidgetContext::scope(world, |ctx| ctx.draw(ui, node))
    }
}
