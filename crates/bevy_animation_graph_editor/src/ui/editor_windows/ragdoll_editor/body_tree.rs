use bevy::{
    asset::{Assets, Handle},
    ecs::world::World,
};
use bevy_animation_graph::core::ragdoll::definition::Ragdoll;

use crate::{
    tree::{RagdollTreeRenderer, Tree, TreeResult},
    ui::{core::EditorWindowContext, editor_windows::ragdoll_editor::RagdollEditorAction},
};

pub struct BodyTree<'a, 'b> {
    pub ragdoll: Handle<Ragdoll>,
    pub world: &'a mut World,
    pub ctx: &'a mut EditorWindowContext<'b>,
}

impl BodyTree<'_, '_> {
    pub fn draw(self, ui: &mut egui::Ui) {
        self.world
            .resource_scope::<Assets<Ragdoll>, _>(|_world, ragdoll_assets| {
                let Some(ragdoll) = ragdoll_assets.get(&self.ragdoll) else {
                    return;
                };

                // Tree, assemble!
                let response =
                    Tree::ragdoll_tree(ragdoll).picker_selector(ui, RagdollTreeRenderer {});

                match response {
                    TreeResult::Leaf(_, response) | TreeResult::Node(_, response) => {
                        // self.hovered = response.hovered;
                        // if let Some(clicked) = response.clicked {
                        //     self.selected = Some(clicked);
                        //     self.edit_buffers.clear();
                        // }
                        if let Some(clicked_node) = response.clicked {
                            self.ctx
                                .window_action(RagdollEditorAction::SelectNode(clicked_node));
                        }
                    }
                    TreeResult::None => {}
                }
            });
    }
}
