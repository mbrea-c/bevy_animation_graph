use bevy::ecs::{reflect::AppTypeRegistry, world::World};
use bevy_animation_graph::prelude::AnimationNode;

use crate::ui::node_editors::{ReflectEditable, reflect_editor::ReflectNodeEditor};

pub struct AnimationNodeWidget<'a> {
    pub node: &'a mut AnimationNode,
    pub id_hash: egui::Id,
    pub world: &'a mut World,
}

impl<'a> AnimationNodeWidget<'a> {
    pub fn new_salted(
        node: &'a mut AnimationNode,
        world: &'a mut World,
        salt: impl std::hash::Hash,
    ) -> Self {
        Self {
            node,
            id_hash: egui::Id::new(salt),
            world,
        }
    }
}

impl<'a> egui::Widget for AnimationNodeWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.push_id(self.id_hash, |ui| {
            let response = ui.text_edit_singleline(&mut self.node.name);

            let editor = if let Some(editable) = self
                .world
                .resource::<AppTypeRegistry>()
                .0
                .clone()
                .read()
                .get_type_data::<ReflectEditable>(self.node.inner.type_id())
            {
                (editable.get_editor)(self.node.inner.as_ref())
            } else {
                Box::new(ReflectNodeEditor)
            };

            let inner_edit_response = editor.show_dyn(ui, self.world, self.node.inner.as_mut());

            response | inner_edit_response
        })
        .inner
    }
}
