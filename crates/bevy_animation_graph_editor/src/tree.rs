use bevy::{
    core::Name,
    ecs::{entity::Entity, world::World},
    hierarchy::Children,
};

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

pub enum TreeResult<I, L> {
    Leaf(L),
    Node(I),
    None,
}

impl<I, L> TreeResult<I, L> {
    // We implement several functions that work similarly to Option<T>

    pub fn or(self, other: Self) -> Self {
        match self {
            TreeResult::None => other,
            _ => self,
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
            .unwrap_or(vec![]);
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
