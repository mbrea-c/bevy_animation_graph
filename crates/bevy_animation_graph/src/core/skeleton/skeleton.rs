use crate::core::{animation_clip::EntityPath, id::BoneId};
use bevy::{asset::Asset, platform::collections::HashMap, reflect::Reflect};
use std::fmt::Debug;

#[derive(Asset, Reflect, Default)]
pub struct Skeleton {
    root: BoneId,
    id_to_path: HashMap<BoneId, EntityPath>,
    children_map: HashMap<BoneId, Vec<BoneId>>,
    parent_map: HashMap<BoneId, BoneId>,
}

impl Skeleton {
    pub fn add_bone(&mut self, path: EntityPath) {
        if path.parts.is_empty() {
            panic!(
                "Cannot have a bone path with length 0! Something must be wrong in the skeleton asset loader..."
            );
        }

        let maybe_parent = path.parent();
        let id = path.id();

        self.id_to_path.insert(id, path);

        if !self.children_map.contains_key(&id) {
            self.children_map.insert(id, vec![]);
        }

        if let Some(parent) = maybe_parent {
            let parent_id = parent.id();
            if let Some(parent_children) = self.children_map.get_mut(&parent_id) {
                parent_children.push(id);
            } else {
                self.children_map.insert(parent_id, vec![id]);
            }
            self.parent_map.insert(id, parent_id);
        } else {
            let parent = EntityPath::default();
            let parent_id = parent.id();
            if let Some(parent_children) = self.children_map.get_mut(&parent_id) {
                parent_children.push(id);
            } else {
                self.children_map.insert(parent_id, vec![id]);
            }

            if !self.id_to_path.contains_key(&parent_id) {
                self.id_to_path.insert(parent_id, parent);
            }
            // self.parent_map.insert(id, parent_id);
        }
    }

    pub fn set_root(&mut self, id: BoneId) {
        self.root = id;
    }

    pub fn root(&self) -> BoneId {
        self.root
    }

    pub fn parent(&self, id: &BoneId) -> Option<BoneId> {
        self.parent_map.get(id).copied()
    }

    pub fn children(&self, id: BoneId) -> Vec<BoneId> {
        self.children_map.get(&id).cloned().unwrap_or_default()
    }

    /// Given an `AnimationTargetId`, returns its path or None if the id is not in this skeleton
    pub fn id_to_path(&self, id: BoneId) -> Option<EntityPath> {
        self.id_to_path.get(&id).cloned()
    }

    pub fn has_id(&self, id: &BoneId) -> bool {
        self.id_to_path.contains_key(id)
    }

    /// Given an `EntityPath`, returns its id or None if the path is not in this skeleton. Note
    /// that for an unconditional conversion from path to id, you can call instead
    /// `EntityPath::id`.
    pub fn path_to_id(&self, path: EntityPath) -> Option<BoneId> {
        let id = path.id();

        self.id_to_path.contains_key(&id).then_some(id)
    }

    fn indent(f: &mut std::fmt::Formatter<'_>, level: u32) -> std::fmt::Result {
        if level == 0 {
            return Ok(());
        }
        for _ in 0..(level - 1) {
            write!(f, "┃ ")?;
        }
        write!(f, "┣━")?;
        Ok(())
    }

    fn fmt_level(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        level: u32,
        parent: BoneId,
    ) -> std::fmt::Result {
        let children = self.children_map.get(&parent).unwrap();
        for child in children.iter() {
            Self::indent(f, level)?;
            writeln!(
                f,
                "🦴 {:?} [{:?}]",
                self.id_to_path.get(child).unwrap().to_slashed_string(),
                child
            )?;
            self.fmt_level(f, level + 1, *child)?;
        }
        Ok(())
    }
}

impl Debug for Skeleton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Skeleton hierarchy:")?;
        self.fmt_level(f, 0, EntityPath::default().id())
    }
}
