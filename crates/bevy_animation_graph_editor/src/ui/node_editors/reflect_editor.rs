use bevy::ecs::world::World;
use bevy_animation_graph::prelude::{NodeLike, ReflectEditProxy};

use crate::ui::{node_editors::DynNodeEditor, utils::using_inspector_env};

#[derive(Default)]
pub struct ReflectNodeEditor;

impl DynNodeEditor for ReflectNodeEditor {
    fn show_dyn(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        node: &mut dyn NodeLike,
    ) -> egui::Response {
        let mut response = ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
        let type_id = node.as_any().type_id();
        let changed = using_inspector_env(world, |mut env| {
            if let Some(edit_proxy) = env.type_registry.get_type_data::<ReflectEditProxy>(type_id) {
                let mut proxy = (edit_proxy.to_proxy)(node);
                env.ui_for_reflect_with_options(proxy.as_partial_reflect_mut(), ui, ui.id(), &())
            } else {
                env.ui_for_reflect(node.as_partial_reflect_mut(), ui)
            }
        });

        if changed {
            response.mark_changed();
        }

        response
    }
}
