use bevy::ecs::{entity::Entity, hierarchy::Children, name::Name, world::World};
use bevy_animation_graph::core::{
    id::BoneId,
    ragdoll::definition::{BodyId, ColliderId, JointId, Ragdoll},
    skeleton::Skeleton,
};
use egui::Sense;

use crate::{icons, ui::utils::collapsing::Collapser};

pub struct Tree<I, L>(pub Vec<TreeInternal<I, L>>);
impl<I, T> Default for Tree<I, T> {
    fn default() -> Self {
        Self(vec![])
    }
}

pub enum TreeInternal<I, L> {
    Leaf(String, L),
    Node(String, I, Vec<TreeInternal<I, L>>),
}

#[derive(Default)]
pub enum TreeResult<I, L, O = ()> {
    Leaf(L, O),
    Node(I, O),
    #[default]
    None,
}

pub trait TreeResponse {
    fn combine(&self, other: &Self) -> Self;
}

impl TreeResponse for () {
    fn combine(&self, _: &Self) -> Self {}
}

impl<I, L, O: TreeResponse> TreeResult<I, L, O> {
    // We implement several functions that work similarly to Option<T>

    pub fn or(self, other: Self) -> Self {
        match self {
            TreeResult::None => other,
            TreeResult::Leaf(data, response) => match other {
                TreeResult::None => TreeResult::Leaf(data, response),
                TreeResult::Leaf(_, other_response) | TreeResult::Node(_, other_response) => {
                    TreeResult::Leaf(data, response.combine(&other_response))
                }
            },
            TreeResult::Node(data, response) => match other {
                TreeResult::Leaf(_, other_response) | TreeResult::Node(_, other_response) => {
                    TreeResult::Node(data, response.combine(&other_response))
                }
                TreeResult::None => TreeResult::Node(data, response),
            },
        }
    }
}

pub trait TreeRenderer<I, L, O> {
    fn render_inner(
        &self,
        label: &str,
        data: &I,
        children: &[TreeInternal<I, L>],
        ui: &mut egui::Ui,
        render_child: impl Fn(&TreeInternal<I, L>, &mut egui::Ui) -> TreeResult<I, L, O>,
    ) -> TreeResult<I, L, O>;

    fn render_leaf(&self, label: &str, data: &L, ui: &mut egui::Ui) -> TreeResult<I, L, O>;
}

impl<I, L> Tree<I, L> {
    pub fn picker_selector<O: TreeResponse>(
        &self,
        ui: &mut egui::Ui,
        renderer: impl TreeRenderer<I, L, O> + Copy,
    ) -> TreeResult<I, L, O> {
        self.0
            .iter()
            .map(|i| Self::picker_selector_internal(i, ui, renderer))
            .reduce(|l, r| l.or(r))
            .unwrap_or_default()
    }

    pub fn picker_selector_internal<O: TreeResponse>(
        internal: &TreeInternal<I, L>,
        ui: &mut egui::Ui,
        renderer: impl TreeRenderer<I, L, O> + Copy,
    ) -> TreeResult<I, L, O> {
        match internal {
            TreeInternal::Leaf(label, data) => renderer.render_leaf(label, data, ui),
            TreeInternal::Node(label, data, tree_internals) => {
                renderer.render_inner(label, data, tree_internals, ui, |internal, ui| {
                    Self::picker_selector_internal(internal, ui, renderer)
                })
            }
        }
    }
}

impl<T> Tree<(), T> {
    pub fn insert(&mut self, mut parts: Vec<String>, value: T) {
        let Some(leaf_name) = parts.pop() else {
            return;
        };

        let mut branch: &mut Vec<TreeInternal<(), T>> = &mut self.0;
        for part in parts {
            if branch.iter().any(|p| match p {
                TreeInternal::Node(p, _, _) => p == &part,
                _ => false,
            }) {
                let b = branch
                    .iter_mut()
                    .find_map(|p| match p {
                        TreeInternal::Node(p, _, b) => (p == &part).then_some(b),
                        _ => None,
                    })
                    .unwrap();
                branch = b;
            } else {
                branch.push(TreeInternal::Node(part, (), vec![]));
                branch = match branch.last_mut().unwrap() {
                    TreeInternal::Node(_, _, b) => b,
                    _ => unreachable!(),
                };
            }
        }

        branch.push(TreeInternal::Leaf(leaf_name, value));
    }
}

impl Tree<(Entity, Vec<Name>), (Entity, Vec<Name>)> {
    /// Returns tree representing the parent/child hierarchy with
    /// the given entity as the root.
    pub fn entity_tree(world: &mut World, entity: Entity) -> Self {
        Tree(vec![Self::entity_subtree(world, entity, vec![])])
    }

    fn entity_subtree(
        world: &mut World,
        entity: Entity,
        path_to_parent: Vec<Name>,
    ) -> TreeInternal<(Entity, Vec<Name>), (Entity, Vec<Name>)> {
        let mut name_query = world.query::<&Name>();
        let mut children_query = world.query::<&Children>();

        let name_path = name_query.get(world, entity).cloned();
        let name = format!(
            "{} ({:?})",
            name_path.clone().unwrap_or_else(|_| "".into()),
            entity
        );
        let name_path: Name = name_path.unwrap_or_else(|_| Name::new(""));
        let mut path_to_entity = path_to_parent.clone();
        path_to_entity.push(name_path);

        let children = children_query
            .get(world, entity)
            .map(|c| c.into_iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        if children.is_empty() {
            TreeInternal::Leaf(name, (entity, path_to_entity))
        } else {
            let mut branch = vec![];
            for child in children {
                let tree = Self::entity_subtree(world, child, path_to_entity.clone());
                branch.push(tree);
            }
            TreeInternal::Node(name, (entity, path_to_entity), branch)
        }
    }
}

#[derive(Clone, Hash, PartialEq)]
pub struct SkeletonNode {
    pub bone_id: BoneId,
}

impl Tree<SkeletonNode, SkeletonNode> {
    pub fn skeleton_tree(skeleton: &Skeleton) -> Self {
        Tree(vec![Self::skeleton_subtree(skeleton, skeleton.root())])
    }

    fn skeleton_subtree(
        skeleton: &Skeleton,
        starting_at: BoneId,
    ) -> TreeInternal<SkeletonNode, SkeletonNode> {
        let path = skeleton.id_to_path(starting_at).unwrap();
        let children = skeleton.children(starting_at);

        if children.is_empty() {
            TreeInternal::Leaf(
                path.last().unwrap_or("ROOT".into()).to_string(),
                SkeletonNode {
                    bone_id: starting_at,
                },
            )
        } else {
            TreeInternal::Node(
                path.last().unwrap_or("ROOT".into()).to_string(),
                SkeletonNode {
                    bone_id: starting_at,
                },
                children
                    .into_iter()
                    .map(|c| Self::skeleton_subtree(skeleton, c))
                    .collect(),
            )
        }
    }
}

#[derive(Clone, Copy)]
pub struct SkeletonTreeRenderer {}

pub struct SkeletonResponse {
    pub hovered: Option<BoneId>,
    pub clicked: Option<BoneId>,
}

impl TreeResponse for SkeletonResponse {
    fn combine(&self, other: &Self) -> Self {
        SkeletonResponse {
            hovered: self.hovered.or(other.hovered),
            clicked: self.clicked.or(other.clicked),
        }
    }
}

impl TreeRenderer<SkeletonNode, SkeletonNode, SkeletonResponse> for SkeletonTreeRenderer {
    fn render_inner(
        &self,
        label: &str,
        data: &SkeletonNode,
        children: &[TreeInternal<SkeletonNode, SkeletonNode>],
        ui: &mut egui::Ui,
        render_child: impl Fn(
            &TreeInternal<SkeletonNode, SkeletonNode>,
            &mut egui::Ui,
        ) -> TreeResult<SkeletonNode, SkeletonNode, SkeletonResponse>,
    ) -> TreeResult<SkeletonNode, SkeletonNode, SkeletonResponse> {
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
        data: &SkeletonNode,
        ui: &mut egui::Ui,
    ) -> TreeResult<SkeletonNode, SkeletonNode, SkeletonResponse> {
        let response = ui
            .horizontal(|ui| {
                let image = icons::BONE;
                let color = ui.visuals().text_color();

                ui.add(egui::Image::new(image).tint(color).sense(Sense::click()))
                    | ui.add(egui::Label::new(label).sense(Sense::click()))
            })
            .inner;

        TreeResult::Leaf(
            data.clone(),
            SkeletonResponse {
                hovered: response.hovered().then_some(data.bone_id),
                clicked: response.clicked().then_some(data.bone_id),
            },
        )
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum RagdollNode {
    Body(BodyId),
    Collider(ColliderId),
    Joint(JointId),
}

impl Tree<RagdollNode, RagdollNode> {
    pub fn ragdoll_tree(ragdoll: &Ragdoll) -> Self {
        Tree(
            ragdoll
                .bodies
                .values()
                .map(|b| Self::ragdoll_body_subtree(ragdoll, b.id))
                .chain(
                    ragdoll
                        .joints
                        .values()
                        .map(|j| Self::ragdoll_joint_subtree(ragdoll, j.id)),
                )
                .collect(),
        )
    }

    fn ragdoll_body_subtree(
        ragdoll: &Ragdoll,
        body_id: BodyId,
    ) -> TreeInternal<RagdollNode, RagdollNode> {
        let body = ragdoll.get_body(body_id).unwrap();
        let children = body.colliders.iter().copied().collect::<Vec<_>>();

        let label = if body.label.is_empty() {
            format!("{:?}", body.id)
        } else {
            body.label.clone()
        };

        if children.is_empty() {
            TreeInternal::Leaf(label, RagdollNode::Body(body_id))
        } else {
            TreeInternal::Node(
                label,
                RagdollNode::Body(body_id),
                children
                    .into_iter()
                    .map(|c| Self::ragdoll_collider_subtree(ragdoll, c))
                    .collect(),
            )
        }
    }

    fn ragdoll_collider_subtree(
        ragdoll: &Ragdoll,
        collider_id: ColliderId,
    ) -> TreeInternal<RagdollNode, RagdollNode> {
        let collider = ragdoll.get_collider(collider_id).unwrap();
        TreeInternal::Leaf(
            if collider.label.is_empty() {
                format!("{:?}", collider.id)
            } else {
                collider.label.clone()
            },
            RagdollNode::Collider(collider_id),
        )
    }

    fn ragdoll_joint_subtree(
        ragdoll: &Ragdoll,
        joint_id: JointId,
    ) -> TreeInternal<RagdollNode, RagdollNode> {
        let joint = ragdoll.get_joint(joint_id).unwrap();
        let label = if joint.label.is_empty() {
            format!("{:?}", joint.id)
        } else {
            joint.label.clone()
        };
        TreeInternal::Leaf(label, RagdollNode::Joint(joint_id))
    }
}
