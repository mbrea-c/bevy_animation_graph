use super::{
    animation_clip::EntityPath,
    context::PassContext,
    pose::{BoneId, BonePose, Pose},
    skeleton::Skeleton,
};
use bevy::{ecs::entity::Entity, transform::components::Transform};
use std::collections::VecDeque;

pub trait SpaceConversion {
    // fn bone_to_character(&self, data: &BonePoseFrame) -> CharacterPoseFrame;
    // fn bone_to_global(&self, data: &BonePoseFrame) -> GlobalPoseFrame;
    // fn character_to_bone(&self, data: &CharacterPoseFrame) -> BonePoseFrame;
    // fn character_to_global(&self, data: &CharacterPoseFrame) -> GlobalPoseFrame;
    // fn global_to_bone(&self, data: &GlobalPoseFrame) -> BonePoseFrame;
    // fn global_to_character(&self, data: &GlobalPoseFrame) -> CharacterPoseFrame;

    /// Given a transform in a space relative to a given bone, convert it into a space
    /// relative to a descendant bone.
    ///
    /// NOTE: data should be in bone space
    ///
    /// ### Panics
    /// Panics if `source` is not an ancestor of `target`.
    fn change_bone_space_down(
        &self,
        transform: Transform,
        data: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        source: BoneId,
        target: BoneId,
    ) -> Transform;

    /// Given a transform in a space relative to a given bone, convert it into a space
    /// relative to an ancestor bone.
    ///
    /// NOTE: data should be in bone space
    ///
    /// ### Panics
    /// Panics if `source` is not an ancestor of `target`.
    fn change_bone_space_up(
        &self,
        transform: Transform,
        data: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        source: BoneId,
        target: BoneId,
    ) -> Transform;

    /// Given a transform in a space relative to the root bone, convert it into a space
    /// relative to a descendant bone.
    ///
    /// NOTE: data should be in bone space
    ///
    /// ### Panics
    /// Panics if `source` is not an ancestor of `target`.
    fn root_to_bone_space(
        &self,
        transform: Transform,
        data: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform;

    fn global_to_bone_space(
        &self,
        transform: Transform,
        data: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform;

    fn transform_global_to_character(&self, transform: Transform) -> Transform;

    /// Returns transform of bone in character space
    fn character_transform_of_bone(
        &self,
        data: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform;

    /// Returns transform of bone in character space
    fn global_transform_of_bone(
        &self,
        data: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform;

    /// Extends the skeleton to include all bones in the hierarchy
    ///
    /// ### Important
    /// **The pose should be in bone space**
    fn extend_skeleton_bone(&self, data: &Pose) -> Pose;
}

impl SpaceConversion for PassContext<'_> {
    // fn bone_to_character(&self, data: &BonePoseFrame) -> CharacterPoseFrame {
    //     let root_name = self.resources.names_query.get(self.root_entity).unwrap();
    //     let root_path = EntityPath {
    //         parts: vec![root_name.clone()],
    //     };
    //     let root_transform_frame = ValueFrame {
    //         prev: Transform::IDENTITY,
    //         prev_timestamp: f32::MIN,
    //         next: Transform::IDENTITY,
    //         next_timestamp: f32::MAX,
    //         prev_is_wrapped: true,
    //         next_is_wrapped: true,
    //     };

    //     let root_children = self.resources.children_query.get(self.root_entity).unwrap();

    //     let mut character_transforms: HashMap<EntityPath, ValueFrame<Transform>> = HashMap::new();
    //     let mut queue: VecDeque<(Entity, EntityPath, ValueFrame<Transform>)> = VecDeque::new();

    //     for child in root_children {
    //         queue.push_back((*child, root_path.clone(), root_transform_frame.clone()));
    //     }

    //     while !queue.is_empty() {
    //         let (entity, parent_path, parent_transform_frame) = queue.pop_front().unwrap();
    //         // --- Compute the updated transform frame
    //         // -------------------------------------------------------
    //         // First, build the entity path for the current entity
    //         let entity_name = self.resources.names_query.get(entity).unwrap();
    //         let entity_path = parent_path.child(entity_name.clone());

    //         // Get the entity's current local transform
    //         let (entity_transform, _) = self.resources.transform_query.get(entity).unwrap();
    //         let inner_data = data.inner_ref();
    //         // Get the corresponding bone frame
    //         let bone_frame: BoneFrame = if inner_data.paths.contains_key(&entity_path) {
    //             let bone_id = inner_data.paths.get(&entity_path).unwrap();
    //             inner_data.bones[*bone_id].clone()
    //         } else {
    //             BoneFrame::default()
    //         };

    //         // Obtain a merged local transform frame
    //         let local_transform_frame = ValueFrame {
    //             prev: *entity_transform,
    //             prev_timestamp: f32::MIN,
    //             next: *entity_transform,
    //             next_timestamp: f32::MAX,
    //             prev_is_wrapped: true,
    //             next_is_wrapped: true,
    //         };
    //         let local_transform_frame =
    //             bone_frame.to_transform_frame_linear_with_base_frame(local_transform_frame);

    //         let character_transform_frame = parent_transform_frame
    //             .merge_linear(&local_transform_frame, |parent, child| *child * *parent);
    //         character_transforms.insert(entity_path.clone(), character_transform_frame.clone());

    //         if let Ok(children) = self.resources.children_query.get(entity) {
    //             for child in children {
    //                 queue.push_back((
    //                     *child,
    //                     entity_path.clone(),
    //                     character_transform_frame.clone(),
    //                 ));
    //             }
    //         }
    //         // -------------------------------------------------------
    //     }

    //     // --- Build character pose frame
    //     // ---
    //     // --- This involves building a bone frame for each bone
    //     // --- frame in the existing data using the computed
    //     // --- character transforms
    //     // -------------------------------------------------------
    //     let mut final_pose_frame = CharacterPoseFrame::default();
    //     let inner_character_frame = final_pose_frame.inner_mut();

    //     for (path, bone_id) in data.inner_ref().paths.iter() {
    //         let local_bone_frame = &data.inner_ref().bones[*bone_id];
    //         let character_transform_frame = character_transforms.get(path).unwrap();
    //         let character_translation_frame = character_transform_frame.map(|t| t.translation);
    //         let character_rotation_frame = character_transform_frame.map(|t| t.rotation);
    //         let character_scale_frame = character_transform_frame.map(|t| t.scale);

    //         let character_bone_frame = BoneFrame {
    //             rotation: Some(character_rotation_frame),
    //             translation: Some(character_translation_frame),
    //             scale: Some(character_scale_frame),
    //             weights: local_bone_frame.weights.clone(),
    //         };

    //         inner_character_frame.add_bone(character_bone_frame, path.clone());
    //     }
    //     // -------------------------------------------------------

    //     final_pose_frame
    // }

    // fn bone_to_global(&self, data: &BonePoseFrame) -> GlobalPoseFrame {
    //     let character_pose_frame = self.bone_to_character(data);
    //     self.character_to_global(&character_pose_frame)
    // }

    // fn character_to_bone(&self, data: &CharacterPoseFrame) -> BonePoseFrame {
    //     let root_name = self.resources.names_query.get(self.root_entity).unwrap();
    //     let root_path = EntityPath {
    //         parts: vec![root_name.clone()],
    //     };
    //     let root_transform_frame = ValueFrame {
    //         prev: Transform::IDENTITY,
    //         prev_timestamp: f32::MIN,
    //         next: Transform::IDENTITY,
    //         next_timestamp: f32::MAX,
    //         prev_is_wrapped: true,
    //         next_is_wrapped: true,
    //     };

    //     let root_children = self.resources.children_query.get(self.root_entity).unwrap();

    //     let mut bone_transforms: HashMap<EntityPath, ValueFrame<Transform>> = HashMap::new();
    //     let mut queue: VecDeque<(
    //         Entity,
    //         EntityPath,
    //         ValueFrame<Transform>,
    //         ValueFrame<Transform>,
    //     )> = VecDeque::new();

    //     for child in root_children {
    //         queue.push_back((
    //             *child,
    //             root_path.clone(),
    //             root_transform_frame.clone(),
    //             root_transform_frame.clone(),
    //         ));
    //     }

    //     while !queue.is_empty() {
    //         let (entity, parent_path, parent_transform_frame, parent_inverse_transform_frame) =
    //             queue.pop_front().unwrap();
    //         // --- Compute the updated transform frame
    //         // -------------------------------------------------------
    //         // First, build the entity path for the current entity
    //         let entity_name = self.resources.names_query.get(entity).unwrap();
    //         let entity_path = parent_path.child(entity_name.clone());

    //         // Get the entity's current local transform (in parent bone space)
    //         let (entity_transform, _) = self.resources.transform_query.get(entity).unwrap();
    //         let inner_data = data.inner_ref();
    //         // Get the corresponding bone frame in character space
    //         let bone_frame: BoneFrame = if inner_data.paths.contains_key(&entity_path) {
    //             let bone_id = inner_data.paths.get(&entity_path).unwrap();
    //             inner_data.bones[*bone_id].clone()
    //         } else {
    //             BoneFrame {
    //                 translation: Some(parent_transform_frame.map(|t| t.translation)),
    //                 rotation: Some(parent_transform_frame.map(|t| t.rotation)),
    //                 scale: Some(parent_transform_frame.map(|t| t.scale)),
    //                 ..Default::default()
    //             }
    //         };

    //         // Obtain a merged character transform frame
    //         let character_transform_frame = ValueFrame {
    //             prev: *entity_transform,
    //             prev_timestamp: f32::MIN,
    //             next: *entity_transform,
    //             next_timestamp: f32::MAX,
    //             prev_is_wrapped: true,
    //             next_is_wrapped: true,
    //         }
    //         .merge_linear(&parent_transform_frame, |child, parent| *child * *parent);

    //         let character_transform_frame =
    //             bone_frame.to_transform_frame_linear_with_base_frame(character_transform_frame);

    //         let bone_transform_frame = parent_inverse_transform_frame
    //             .merge_linear(&character_transform_frame, |parent, child| *child * *parent);
    //         bone_transforms.insert(entity_path.clone(), bone_transform_frame.clone());

    //         if let Ok(children) = self.resources.children_query.get(entity) {
    //             for child in children {
    //                 queue.push_back((
    //                     *child,
    //                     entity_path.clone(),
    //                     character_transform_frame.clone(),
    //                     character_transform_frame
    //                         .map(|t| Transform::from_matrix(t.compute_matrix().inverse())),
    //                 ));
    //             }
    //         }
    //         // -------------------------------------------------------
    //     }

    //     // --- Build character pose frame
    //     // ---
    //     // --- This involves building a bone frame for each bone
    //     // --- frame in the existing data using the computed
    //     // --- character transforms
    //     // -------------------------------------------------------
    //     let mut final_pose_frame = BonePoseFrame::default();
    //     let inner_character_frame = final_pose_frame.inner_mut();

    //     for (path, bone_id) in data.inner_ref().paths.iter() {
    //         let local_bone_frame = &data.inner_ref().bones[*bone_id];
    //         let character_transform_frame = bone_transforms.get(path).unwrap();
    //         let character_translation_frame = character_transform_frame.map(|t| t.translation);
    //         let character_rotation_frame = character_transform_frame.map(|t| t.rotation);
    //         let character_scale_frame = character_transform_frame.map(|t| t.scale);

    //         let character_bone_frame = BoneFrame {
    //             rotation: Some(character_rotation_frame),
    //             translation: Some(character_translation_frame),
    //             scale: Some(character_scale_frame),
    //             weights: local_bone_frame.weights.clone(),
    //         };

    //         inner_character_frame.add_bone(character_bone_frame, path.clone());
    //     }
    //     // -------------------------------------------------------

    //     final_pose_frame
    // }

    // fn character_to_global(&self, data: &CharacterPoseFrame) -> GlobalPoseFrame {
    //     let (_, root_global_transform) = self
    //         .resources
    //         .transform_query
    //         .get(self.root_entity)
    //         .unwrap();
    //     let root_global_transform = root_global_transform.compute_transform();

    //     // --- Build character pose frame
    //     // ---
    //     // --- This involves building a bone frame for each bone
    //     // --- frame in the existing data using the computed
    //     // --- inverse root transform
    //     // -------------------------------------------------------
    //     let mut final_pose_frame = GlobalPoseFrame::default();
    //     let inner_global_frame = final_pose_frame.inner_mut();

    //     for (path, bone_id) in data.inner_ref().paths.iter() {
    //         let global_bone_frame = &data.inner_ref().bones[*bone_id];

    //         let global_bone_frame = BoneFrame {
    //             rotation: global_bone_frame
    //                 .rotation
    //                 .as_ref()
    //                 .map(|frame| frame.map(|r| root_global_transform.rotation * *r)),
    //             translation: global_bone_frame
    //                 .translation
    //                 .as_ref()
    //                 .map(|frame| frame.map(|t| root_global_transform * *t)),
    //             scale: global_bone_frame
    //                 .scale
    //                 .as_ref()
    //                 .map(|frame| frame.map(|s| root_global_transform.scale * *s)),
    //             weights: global_bone_frame.weights.clone(),
    //         };

    //         inner_global_frame.add_bone(global_bone_frame, path.clone());
    //     }
    //     // -------------------------------------------------------

    //     final_pose_frame
    // }

    // fn global_to_bone(&self, data: &GlobalPoseFrame) -> BonePoseFrame {
    //     let character_pose_frame = self.global_to_character(data);
    //     self.character_to_bone(&character_pose_frame)
    // }

    // fn global_to_character(&self, data: &GlobalPoseFrame) -> CharacterPoseFrame {
    //     let (_, root_global_transform) = self
    //         .resources
    //         .transform_query
    //         .get(self.root_entity)
    //         .unwrap();
    //     let inverse_global_transform =
    //         Transform::from_matrix(root_global_transform.compute_matrix().inverse());

    //     // --- Build character pose frame
    //     // ---
    //     // --- This involves building a bone frame for each bone
    //     // --- frame in the existing data using the computed
    //     // --- inverse root transform
    //     // -------------------------------------------------------
    //     let mut final_pose_frame = CharacterPoseFrame::default();
    //     let inner_character_frame = final_pose_frame.inner_mut();

    //     for (path, bone_id) in data.inner_ref().paths.iter() {
    //         let global_bone_frame = &data.inner_ref().bones[*bone_id];

    //         let character_bone_frame = BoneFrame {
    //             rotation: global_bone_frame
    //                 .rotation
    //                 .as_ref()
    //                 .map(|frame| frame.map(|r| inverse_global_transform.rotation * *r)),
    //             translation: global_bone_frame
    //                 .translation
    //                 .as_ref()
    //                 .map(|frame| frame.map(|t| inverse_global_transform * *t)),
    //             scale: global_bone_frame
    //                 .scale
    //                 .as_ref()
    //                 .map(|frame| frame.map(|s| inverse_global_transform.scale * *s)),
    //             weights: global_bone_frame.weights.clone(),
    //         };

    //         inner_character_frame.add_bone(character_bone_frame, path.clone());
    //     }
    //     // -------------------------------------------------------

    //     final_pose_frame
    // }

    fn change_bone_space_down(
        &self,
        transform: Transform,
        data: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        source: BoneId,
        target: BoneId,
    ) -> Transform {
        let mut curr_path = target;
        let mut curr_transform = Transform::IDENTITY;

        while curr_path != source {
            let bone_frame = if data.paths.contains_key(&curr_path) {
                let bone_id = data.paths.get(&curr_path).unwrap();
                data.bones[*bone_id].clone()
            } else {
                BonePose::default()
            };
            let curr_entity = self.entity_map.get(&curr_path).unwrap();
            let curr_local_transform = self.resources.transform_query.get(*curr_entity).unwrap().0;
            let merged_local_transform = bone_frame.to_transform_with_base(*curr_local_transform);

            curr_transform = merged_local_transform * curr_transform;
            curr_path = skeleton.parent(&curr_path).unwrap();
        }

        Transform::from_matrix(curr_transform.compute_matrix().inverse()) * transform
    }

    fn change_bone_space_up(
        &self,
        transform: Transform,
        data: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        source: BoneId,
        target: BoneId,
    ) -> Transform {
        let mut curr_path = source;
        let mut curr_transform = Transform::IDENTITY;

        while curr_path != target {
            let bone_pose: BonePose = if data.paths.contains_key(&curr_path) {
                let bone_id = data.paths.get(&curr_path).unwrap();
                data.bones[*bone_id].clone()
            } else {
                BonePose::default()
            };
            let curr_entity = self.entity_map.get(&curr_path).unwrap();
            let curr_local_transform = self.resources.transform_query.get(*curr_entity).unwrap().0;
            let merged_local_transform = bone_pose.to_transform_with_base(*curr_local_transform);

            curr_transform = merged_local_transform * curr_transform;
            curr_path = skeleton.parent(&curr_path).unwrap();
        }

        curr_transform * transform
    }

    fn root_to_bone_space(
        &self,
        transform: Transform,
        pose: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform {
        let root_name = self.resources.names_query.get(self.root_entity).unwrap();
        let root_path = EntityPath {
            parts: vec![root_name.clone()],
        };

        self.change_bone_space_down(transform, pose, skeleton, root_path.id(), target)
    }

    fn global_to_bone_space(
        &self,
        transform: Transform,
        pose: &Pose, // Should be in bone space
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform {
        let character_transform = self.transform_global_to_character(transform);
        self.root_to_bone_space(character_transform, pose, skeleton, target)
    }

    fn transform_global_to_character(&self, transform: Transform) -> Transform {
        let (_, root_global_transform) = self
            .resources
            .transform_query
            .get(self.root_entity)
            .unwrap();
        let inverse_global_transform =
            Transform::from_matrix(root_global_transform.compute_matrix().inverse());
        inverse_global_transform * transform
    }

    fn character_transform_of_bone(
        &self,
        pose: &Pose,
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform {
        let root_name = self.resources.names_query.get(self.root_entity).unwrap();
        let root_path = EntityPath {
            parts: vec![root_name.clone()],
        };

        self.change_bone_space_up(Transform::IDENTITY, pose, skeleton, target, root_path.id())
    }

    fn global_transform_of_bone(
        &self,
        pose: &Pose,
        skeleton: &Skeleton,
        target: BoneId,
    ) -> Transform {
        let (_, root_transform_global) = self
            .resources
            .transform_query
            .get(self.root_entity)
            .unwrap();
        root_transform_global.compute_transform()
            * self.character_transform_of_bone(pose, skeleton, target)
    }

    fn extend_skeleton_bone(&self, data: &Pose) -> Pose {
        let mut new_pose = data.clone();

        let root_name = self.resources.names_query.get(self.root_entity).unwrap();
        let root_path = EntityPath {
            parts: vec![root_name.clone()],
        };

        let root_children = self.resources.children_query.get(self.root_entity).unwrap();

        let mut queue: VecDeque<(Entity, EntityPath)> = VecDeque::new();

        for child in root_children {
            queue.push_back((*child, root_path.clone()));
        }

        while !queue.is_empty() {
            let (entity, parent_path) = queue.pop_front().unwrap();
            // --- Compute the updated transform frame
            // -------------------------------------------------------
            // First, build the entity path for the current entity
            let entity_name = self.resources.names_query.get(entity).unwrap();
            let entity_path = parent_path.child(entity_name.clone());

            // Get the entity's current local transform
            let (entity_transform, _) = self.resources.transform_query.get(entity).unwrap();
            // Get the corresponding bone frame
            let mut bone_pose = if new_pose.paths.contains_key(&entity_path.id()) {
                let bone_id = new_pose.paths.get(&entity_path.id()).unwrap();
                new_pose.bones[*bone_id].clone()
            } else {
                BonePose::default()
            };

            // Obtain a merged local transform frame

            if bone_pose.translation.is_none() {
                bone_pose.translation = Some(entity_transform.translation);
            }
            if bone_pose.rotation.is_none() {
                bone_pose.rotation = Some(entity_transform.rotation);
            }
            if bone_pose.scale.is_none() {
                bone_pose.scale = Some(entity_transform.scale);
            }

            new_pose.add_bone(bone_pose, entity_path.id());
        }

        new_pose
    }
}
