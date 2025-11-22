use bevy::{asset::Handle, ecs::world::World};
use bevy_animation_graph::core::skeleton::Skeleton;

use super::SelectedItem;
use crate::{
    tree::{SkeletonTreeRenderer, Tree, TreeResult},
    ui::{
        core::LegacyEditorWindowContext, editor_windows::ragdoll_editor::RagdollEditorAction,
        utils::with_assets_all,
    },
};

pub struct BoneTree<'a, 'b> {
    pub skeleton: Handle<Skeleton>,
    pub world: &'a mut World,
    pub ctx: &'a mut LegacyEditorWindowContext<'b>,
}

impl BoneTree<'_, '_> {
    pub fn draw(self, ui: &mut egui::Ui) {
        with_assets_all(self.world, [self.skeleton.id()], |_, [skeleton]| {
            // Tree, assemble!
            let response =
                Tree::skeleton_tree(skeleton).picker_selector(ui, SkeletonTreeRenderer {});
            match response {
                TreeResult::Leaf(_, response) | TreeResult::Node(_, response) => {
                    // self.hovered = response.hovered;
                    // if let Some(clicked) = response.clicked {
                    //     self.selected = Some(clicked);
                    //     self.edit_buffers.clear();
                    // }
                    if let Some(clicked_bone) = response.clicked {
                        self.ctx.window_action(RagdollEditorAction::SelectNode(
                            SelectedItem::Bone(clicked_bone),
                        ));
                    }

                    if let Some(hovered_bone) = response.hovered {
                        self.ctx
                            .window_action(RagdollEditorAction::HoverNode(SelectedItem::Bone(
                                hovered_bone,
                            )));
                    }

                    // for action in response.actions {
                    //     match action {
                    //         RagdollTreeAction::CreateCollider(body_id) => todo!(),
                    //         RagdollTreeAction::CreateBody => {
                    //             self.ctx.editor_actions.dynamic(CreateRagdollBody {
                    //                 ragdoll: self.ragdoll.clone(),
                    //                 body: Body::new(),
                    //             })
                    //         }
                    //         RagdollTreeAction::CreateJoint => todo!(),
                    //         RagdollTreeAction::DeleteBody(body_id) => todo!(),
                    //         RagdollTreeAction::DeleteJoint(joint_id) => todo!(),
                    //         RagdollTreeAction::DeleteCollider(collider_id) => todo!(),
                    //     }
                    // }
                }
                TreeResult::None => {}
            }
        });
    }
}
