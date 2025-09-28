use bevy::{
    asset::{Assets, Handle},
    ecs::world::World,
};
use bevy_animation_graph::core::ragdoll::definition::{Body, BodyId, ColliderId, JointId, Ragdoll};
use egui::Sense;

use crate::{
    icons,
    tree::{RagdollNode, Tree, TreeInternal, TreeRenderer, TreeResponse, TreeResult},
    ui::{
        actions::ragdoll::CreateRagdollBody, core::EditorWindowContext,
        editor_windows::ragdoll_editor::RagdollEditorAction, utils::collapsing::Collapser,
    },
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

                        for action in response.actions {
                            match action {
                                RagdollTreeAction::CreateCollider(body_id) => todo!(),
                                RagdollTreeAction::CreateBody => {
                                    self.ctx.editor_actions.dynamic(CreateRagdollBody {
                                        ragdoll: self.ragdoll.clone(),
                                        body: Body::new(),
                                    })
                                }
                                RagdollTreeAction::CreateJoint => todo!(),
                                RagdollTreeAction::DeleteBody(body_id) => todo!(),
                                RagdollTreeAction::DeleteJoint(joint_id) => todo!(),
                                RagdollTreeAction::DeleteCollider(collider_id) => todo!(),
                            }
                        }
                    }
                    TreeResult::None => {}
                }
            });
    }
}

#[derive(Clone, Copy)]
pub struct RagdollTreeRenderer {}

impl RagdollTreeRenderer {}

pub struct RagdollResponse {
    pub hovered: Option<RagdollNode>,
    pub clicked: Option<RagdollNode>,
    pub actions: Vec<RagdollTreeAction>,
}

impl TreeResponse for RagdollResponse {
    fn combine(&self, other: &Self) -> Self {
        RagdollResponse {
            hovered: self.hovered.or(other.hovered),
            clicked: self.clicked.or(other.clicked),
            actions: self
                .actions
                .clone()
                .into_iter()
                .chain(other.actions.clone())
                .collect(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum RagdollTreeAction {
    CreateCollider(BodyId),
    CreateBody,
    CreateJoint,
    DeleteBody(BodyId),
    DeleteJoint(JointId),
    DeleteCollider(ColliderId),
}

impl TreeRenderer<RagdollNode, RagdollNode, RagdollResponse> for RagdollTreeRenderer {
    fn render_inner(
        &self,
        label: &str,
        data: &RagdollNode,
        children: &[TreeInternal<RagdollNode, RagdollNode>],
        ui: &mut egui::Ui,
        render_child: impl Fn(
            &TreeInternal<RagdollNode, RagdollNode>,
            &mut egui::Ui,
        ) -> TreeResult<RagdollNode, RagdollNode, RagdollResponse>,
    ) -> TreeResult<RagdollNode, RagdollNode, RagdollResponse> {
        let collapsing_response = Collapser::new()
            .with_default_open(true)
            .with_id_salt(data)
            .show(
                ui,
                |ui| self.render_leaf(label, data, ui),
                |ui| {
                    children
                        .iter()
                        .map(|c| render_child(c, ui))
                        .reduce(|l, r| l.or(r))
                        .unwrap_or_default()
                },
            );

        collapsing_response
            .head
            .or(collapsing_response.body.unwrap_or_default())
    }

    fn render_leaf(
        &self,
        label: &str,
        data: &RagdollNode,
        ui: &mut egui::Ui,
    ) -> TreeResult<RagdollNode, RagdollNode, RagdollResponse> {
        let response = ui
            .horizontal(|ui| {
                let image = match data {
                    RagdollNode::Body(_) => icons::BONE,
                    RagdollNode::Collider(_) => icons::BOX,
                    RagdollNode::Joint(_) => icons::JOINT,
                };
                let color = ui.visuals().text_color();

                ui.add(egui::Image::new(image).tint(color).sense(Sense::click()))
                    | ui.add(egui::Label::new(label).sense(Sense::click()))
            })
            .inner;

        let mut actions = Vec::new();
        egui::Popup::context_menu(&response).show(|ui| {
            if ui.button("Add body").clicked() {
                actions.push(RagdollTreeAction::CreateBody);
            }
            if ui.button("Add joint").clicked() {
                actions.push(RagdollTreeAction::CreateJoint);
            }
            match data {
                RagdollNode::Body(body_id) => {
                    if ui.button("Add collider").clicked() {
                        actions.push(RagdollTreeAction::CreateCollider(*body_id));
                    }
                    if ui.button("Delete").clicked() {
                        actions.push(RagdollTreeAction::DeleteBody(*body_id));
                    }
                }
                RagdollNode::Collider(collider_id) => {
                    if ui.button("Delete").clicked() {
                        actions.push(RagdollTreeAction::DeleteCollider(*collider_id));
                    }
                }
                RagdollNode::Joint(joint_id) => {
                    if ui.button("Delete").clicked() {
                        actions.push(RagdollTreeAction::DeleteJoint(*joint_id));
                    }
                }
            }
        });

        TreeResult::Leaf(
            data.clone(),
            RagdollResponse {
                hovered: response.hovered().then_some(*data),
                clicked: response
                    .clicked_by(egui::PointerButton::Primary)
                    .then_some(*data),
                actions,
            },
        )
    }
}
