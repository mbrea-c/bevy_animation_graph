use bevy::ecs::world::World;
use bevy_animation_graph::builtin_nodes::blend_space_node::BlendSpaceNode;

use crate::ui::{
    generic_widgets::list::ListWidget,
    node_editors::{Editable, NodeEditor},
    reflect_lib::ReflectWidgetContext,
};

pub struct BlendSpaceNodeEditor;

impl NodeEditor for BlendSpaceNodeEditor {
    type Target = BlendSpaceNode;

    fn show(
        &self,
        ui: &mut egui::Ui,
        world: &mut World,
        node: &mut Self::Target,
    ) -> egui::Response {
        let mut response = ReflectWidgetContext::scope(world, |ctx| {
            let mut response = ctx.draw(ui, &mut node.mode);
            response |= ctx.draw(ui, &mut node.sync_mode);

            response
        });

        response |= ListWidget::new_salted(&mut node.points, "blend space node point list")
            .ui(ui, |ui, pt| {
                ReflectWidgetContext::scope(world, |ctx| ctx.draw(ui, pt))
            });

        if response.changed() {
            node.refresh_triangulation();
        }

        response
    }
}

impl Editable for BlendSpaceNode {
    type Editor = BlendSpaceNodeEditor;

    fn get_editor(&self) -> Self::Editor {
        BlendSpaceNodeEditor
    }
}
