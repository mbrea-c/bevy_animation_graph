use bevy::{
    asset::{Assets, Handle},
    ecs::world::World,
};
use bevy_animation_graph::core::ragdoll::definition::Ragdoll;

use crate::ui::core::EditorWindowContext;

pub struct BodyTree<'a, 'b> {
    pub ragdoll: Handle<Ragdoll>,
    pub world: &'a mut World,
    pub ctx: &'a mut EditorWindowContext<'b>,
}

impl BodyTree<'_, '_> {
    pub fn draw(self, ui: &mut egui::Ui) {
        self.world
            .resource_scope::<Assets<Ragdoll>, _>(|_world, ragdoll_assets| {
                let Some(_ragdoll) = ragdoll_assets.get(&self.ragdoll) else {
                    return;
                };
            });
    }
}

// pub fn draw_tree_view(
//     &mut self,
//     ui: &mut egui::Ui,
//     world: &mut World,
//     _ctx: &mut EditorWindowContext,
// ) {
//     let Some(target) = &self.target else {
//         ui.centered_and_justified(|ui| {
//             ui.label("No target selected");
//         });
//         return;
//     };
// 
//     egui::ScrollArea::both().show(ui, |ui| {
//         world.resource_scope::<Assets<SkeletonColliders>, _>(|world, skeleton_colliders| {
//             world.resource_scope::<Assets<Skeleton>, _>(|_, skeletons| {
//                 let Some(skeleton_colliders) = skeleton_colliders.get(target) else {
//                     return;
//                 };
//                 let Some(skeleton) = skeletons.get(&skeleton_colliders.skeleton) else {
//                     return;
//                 };
// 
//                 // Tree, assemble!
//                 let response =
//                     Tree::skeleton_tree(skeleton).picker_selector(ui, SkeletonTreeRenderer {});
// 
//                 match response {
//                     TreeResult::Leaf(_, response) | TreeResult::Node(_, response) => {
//                         self.hovered = response.hovered;
//                         if let Some(clicked) = response.clicked {
//                             self.selected = Some(clicked);
//                             self.edit_buffers.clear();
//                         }
//                     }
//                     _ => {}
//                 };
//             })
//         });
//     });
// }
