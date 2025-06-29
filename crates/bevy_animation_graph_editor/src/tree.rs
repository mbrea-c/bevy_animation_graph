use bevy::ecs::{entity::Entity, hierarchy::Children, name::Name, world::World};
use bevy_animation_graph::core::{id::BoneId, skeleton::Skeleton};
use egui::Sense;

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
    fn combine(&self, _: &Self) -> Self {
        ()
    }
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
            .map(|i| self.picker_selector_internal(i, ui, renderer))
            .reduce(|l, r| l.or(r))
            .unwrap_or_default()
    }

    pub fn picker_selector_internal<O: TreeResponse>(
        &self,
        internal: &TreeInternal<I, L>,
        ui: &mut egui::Ui,
        renderer: impl TreeRenderer<I, L, O> + Copy,
    ) -> TreeResult<I, L, O> {
        match internal {
            TreeInternal::Leaf(label, data) => renderer.render_leaf(label, data, ui),
            TreeInternal::Node(label, data, tree_internals) => {
                renderer.render_inner(label, data, tree_internals, ui, |internal, ui| {
                    self.picker_selector_internal(internal, ui, renderer)
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

#[derive(Clone)]
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
        let collapsing_response =
            egui::CollapsingHeader::new(label)
                .default_open(true)
                .show(ui, |ui| {
                    children
                        .iter()
                        .map(|c| render_child(c, ui))
                        .reduce(|l, r| l.or(r))
                        .unwrap_or_default()
                });

        let response = TreeResult::Node(
            data.clone(),
            SkeletonResponse {
                hovered: collapsing_response
                    .header_response
                    .hovered()
                    .then_some(data.bone_id),
                clicked: collapsing_response
                    .header_response
                    .clicked()
                    .then_some(data.bone_id),
            },
        );

        response.or(collapsing_response
            .body_returned
            .map(|r| r)
            .unwrap_or_default())
    }

    fn render_leaf(
        &self,
        label: &str,
        data: &SkeletonNode,
        ui: &mut egui::Ui,
    ) -> TreeResult<SkeletonNode, SkeletonNode, SkeletonResponse> {
        let response = ui.add(egui::Label::new(label).sense(Sense::click()));

        TreeResult::Leaf(
            data.clone(),
            SkeletonResponse {
                hovered: response.hovered().then_some(data.bone_id),
                clicked: response.clicked().then_some(data.bone_id),
            },
        )
    }
}
