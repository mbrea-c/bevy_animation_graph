use std::collections::VecDeque;

use bevy::{ecs::entity::Entity, transform::components::Transform, utils::HashMap};

use super::{
    animation_clip::EntityPath,
    context::PassContext,
    frame::{BoneFrame, BonePoseFrame, CharacterPoseFrame, GlobalPoseFrame, ValueFrame},
};

pub fn bone_to_character(data: &BonePoseFrame, ctx: PassContext) -> CharacterPoseFrame {
    let root_name = ctx.resources.names_query.get(ctx.root_entity).unwrap();
    let root_path = EntityPath {
        parts: vec![root_name.clone()],
    };
    let root_transform_frame = ValueFrame {
        prev: Transform::IDENTITY,
        prev_timestamp: f32::MIN,
        next: Transform::IDENTITY,
        next_timestamp: f32::MAX,
        prev_is_wrapped: true,
        next_is_wrapped: true,
    };

    let root_children = ctx.resources.children_query.get(ctx.root_entity).unwrap();

    let mut character_transforms: HashMap<EntityPath, ValueFrame<Transform>> = HashMap::new();
    let mut queue: VecDeque<(Entity, EntityPath, ValueFrame<Transform>)> = VecDeque::new();

    for child in root_children {
        queue.push_back((*child, root_path.clone(), root_transform_frame.clone()));
    }

    while !queue.is_empty() {
        let (entity, parent_path, parent_transform_frame) = queue.pop_front().unwrap();
        // --- Compute the updated transform frame
        // -------------------------------------------------------
        // First, build the entity path for the current entity
        let entity_name = ctx.resources.names_query.get(entity).unwrap();
        let entity_path = parent_path.child(entity_name.clone());

        // Get the entity's current local transform
        let (entity_transform, _) = ctx.resources.transform_query.get(entity).unwrap();
        let inner_data = data.inner_ref();
        // Get the corresponding bone frame
        let bone_frame: BoneFrame = if inner_data.paths.contains_key(&entity_path) {
            let bone_id = inner_data.paths.get(&entity_path).unwrap();
            inner_data.bones[*bone_id].clone()
        } else {
            BoneFrame::default()
        };

        // Obtain a merged local transform frame
        let mut local_transform_frame = ValueFrame {
            prev: entity_transform.clone(),
            prev_timestamp: f32::MIN,
            next: entity_transform.clone(),
            next_timestamp: f32::MAX,
            prev_is_wrapped: true,
            next_is_wrapped: true,
        };

        if let Some(translation_frame) = &bone_frame.translation {
            local_transform_frame =
                local_transform_frame.merge_linear(translation_frame, |transform, translation| {
                    let mut new_transform = transform.clone();
                    new_transform.translation = *translation;
                    new_transform
                });
        }
        if let Some(rotation_frame) = &bone_frame.rotation {
            local_transform_frame =
                local_transform_frame.merge_linear(rotation_frame, |transform, rotation| {
                    let mut new_transform = transform.clone();
                    new_transform.rotation = *rotation;
                    new_transform
                });
        }
        if let Some(scale_frame) = &bone_frame.scale {
            local_transform_frame =
                local_transform_frame.merge_linear(scale_frame, |transform, scale| {
                    let mut new_transform = transform.clone();
                    new_transform.scale = *scale;
                    new_transform
                });
        }

        let character_transform_frame = parent_transform_frame
            .merge_linear(&local_transform_frame, |parent, child| *child * *parent);
        character_transforms.insert(entity_path.clone(), character_transform_frame.clone());

        if let Ok(children) = ctx.resources.children_query.get(entity) {
            for child in children {
                queue.push_back((
                    *child,
                    entity_path.clone(),
                    character_transform_frame.clone(),
                ));
            }
        }
        // -------------------------------------------------------
    }

    // --- Build character pose frame
    // ---
    // --- This involves building a bone frame for each bone
    // --- frame in the existing data using the computed
    // --- character transforms
    // -------------------------------------------------------
    let mut final_pose_frame = CharacterPoseFrame::default();
    let inner_character_frame = final_pose_frame.inner_mut();

    for (path, bone_id) in data.inner_ref().paths.iter() {
        let local_bone_frame = &data.inner_ref().bones[*bone_id];
        let character_transform_frame = character_transforms.get(path).unwrap();
        let character_translation_frame = character_transform_frame.map(|t| t.translation);
        let character_rotation_frame = character_transform_frame.map(|t| t.rotation);
        let character_scale_frame = character_transform_frame.map(|t| t.scale);

        let character_bone_frame = BoneFrame {
            rotation: Some(character_rotation_frame),
            translation: Some(character_translation_frame),
            scale: Some(character_scale_frame),
            weights: local_bone_frame.weights.clone(),
        };

        inner_character_frame.add_bone(character_bone_frame, path.clone());
    }
    // -------------------------------------------------------

    final_pose_frame
}

pub fn global_to_character(data: &GlobalPoseFrame, ctx: PassContext) -> CharacterPoseFrame {
    let (_, root_global_transform) = ctx.resources.transform_query.get(ctx.root_entity).unwrap();
    let inverse_global_transform =
        Transform::from_matrix(root_global_transform.compute_matrix().inverse());

    // --- Build character pose frame
    // ---
    // --- This involves building a bone frame for each bone
    // --- frame in the existing data using the computed
    // --- inverse root transform
    // -------------------------------------------------------
    let mut final_pose_frame = CharacterPoseFrame::default();
    let inner_character_frame = final_pose_frame.inner_mut();

    for (path, bone_id) in data.inner_ref().paths.iter() {
        let global_bone_frame = &data.inner_ref().bones[*bone_id];

        let character_bone_frame = BoneFrame {
            rotation: global_bone_frame
                .rotation
                .as_ref()
                .map(|frame| frame.map(|r| inverse_global_transform.rotation * *r)),
            translation: global_bone_frame
                .translation
                .as_ref()
                .map(|frame| frame.map(|t| inverse_global_transform * *t)),
            scale: global_bone_frame
                .scale
                .as_ref()
                .map(|frame| frame.map(|s| inverse_global_transform.scale * *s)),
            weights: global_bone_frame.weights.clone(),
        };

        inner_character_frame.add_bone(character_bone_frame, path.clone());
    }
    // -------------------------------------------------------

    final_pose_frame
}

pub fn character_to_global(data: &CharacterPoseFrame, ctx: PassContext) -> GlobalPoseFrame {
    let (_, root_global_transform) = ctx.resources.transform_query.get(ctx.root_entity).unwrap();
    let root_global_transform = root_global_transform.compute_transform();

    // --- Build character pose frame
    // ---
    // --- This involves building a bone frame for each bone
    // --- frame in the existing data using the computed
    // --- inverse root transform
    // -------------------------------------------------------
    let mut final_pose_frame = GlobalPoseFrame::default();
    let inner_global_frame = final_pose_frame.inner_mut();

    for (path, bone_id) in data.inner_ref().paths.iter() {
        let global_bone_frame = &data.inner_ref().bones[*bone_id];

        let global_bone_frame = BoneFrame {
            rotation: global_bone_frame
                .rotation
                .as_ref()
                .map(|frame| frame.map(|r| root_global_transform.rotation * *r)),
            translation: global_bone_frame
                .translation
                .as_ref()
                .map(|frame| frame.map(|t| root_global_transform * *t)),
            scale: global_bone_frame
                .scale
                .as_ref()
                .map(|frame| frame.map(|s| root_global_transform.scale * *s)),
            weights: global_bone_frame.weights.clone(),
        };

        inner_global_frame.add_bone(global_bone_frame, path.clone());
    }
    // -------------------------------------------------------

    final_pose_frame
}

pub fn character_to_bone(data: &CharacterPoseFrame, ctx: PassContext) -> BonePoseFrame {
    let root_name = ctx.resources.names_query.get(ctx.root_entity).unwrap();
    let root_path = EntityPath {
        parts: vec![root_name.clone()],
    };
    let root_transform_frame = ValueFrame {
        prev: Transform::IDENTITY,
        prev_timestamp: f32::MIN,
        next: Transform::IDENTITY,
        next_timestamp: f32::MAX,
        prev_is_wrapped: true,
        next_is_wrapped: true,
    };

    let root_children = ctx.resources.children_query.get(ctx.root_entity).unwrap();

    let mut bone_transforms: HashMap<EntityPath, ValueFrame<Transform>> = HashMap::new();
    let mut queue: VecDeque<(
        Entity,
        EntityPath,
        ValueFrame<Transform>,
        ValueFrame<Transform>,
    )> = VecDeque::new();

    for child in root_children {
        queue.push_back((
            *child,
            root_path.clone(),
            root_transform_frame.clone(),
            root_transform_frame.clone(),
        ));
    }

    while !queue.is_empty() {
        let (entity, parent_path, parent_transform_frame, parent_inverse_transform_frame) =
            queue.pop_front().unwrap();
        // --- Compute the updated transform frame
        // -------------------------------------------------------
        // First, build the entity path for the current entity
        let entity_name = ctx.resources.names_query.get(entity).unwrap();
        let entity_path = parent_path.child(entity_name.clone());

        // Get the entity's current local transform (in parent bone space)
        let (entity_transform, _) = ctx.resources.transform_query.get(entity).unwrap();
        let inner_data = data.inner_ref();
        // Get the corresponding bone frame in character space
        let bone_frame: BoneFrame = if inner_data.paths.contains_key(&entity_path) {
            let bone_id = inner_data.paths.get(&entity_path).unwrap();
            inner_data.bones[*bone_id].clone()
        } else {
            let mut default_frame = BoneFrame::default();
            default_frame.translation = Some(parent_transform_frame.map(|t| t.translation));
            default_frame.rotation = Some(parent_transform_frame.map(|t| t.rotation));
            default_frame.scale = Some(parent_transform_frame.map(|t| t.scale));
            default_frame
        };

        // Obtain a merged character transform frame
        let mut character_transform_frame = ValueFrame {
            prev: entity_transform.clone(),
            prev_timestamp: f32::MIN,
            next: entity_transform.clone(),
            next_timestamp: f32::MAX,
            prev_is_wrapped: true,
            next_is_wrapped: true,
        }
        .merge_linear(&parent_transform_frame, |child, parent| *child * *parent);

        if let Some(translation_frame) = &bone_frame.translation {
            character_transform_frame = character_transform_frame.merge_linear(
                translation_frame,
                |transform, translation| {
                    let mut new_transform = transform.clone();
                    new_transform.translation = *translation;
                    new_transform
                },
            );
        }
        if let Some(rotation_frame) = &bone_frame.rotation {
            character_transform_frame =
                character_transform_frame.merge_linear(rotation_frame, |transform, rotation| {
                    let mut new_transform = transform.clone();
                    new_transform.rotation = *rotation;
                    new_transform
                });
        }
        if let Some(scale_frame) = &bone_frame.scale {
            character_transform_frame =
                character_transform_frame.merge_linear(scale_frame, |transform, scale| {
                    let mut new_transform = transform.clone();
                    new_transform.scale = *scale;
                    new_transform
                });
        }

        let bone_transform_frame = parent_inverse_transform_frame
            .merge_linear(&character_transform_frame, |parent, child| *child * *parent);
        bone_transforms.insert(entity_path.clone(), bone_transform_frame.clone());

        if let Ok(children) = ctx.resources.children_query.get(entity) {
            for child in children {
                queue.push_back((
                    *child,
                    entity_path.clone(),
                    character_transform_frame.clone(),
                    character_transform_frame
                        .map(|t| Transform::from_matrix(t.compute_matrix().inverse())),
                ));
            }
        }
        // -------------------------------------------------------
    }

    // --- Build character pose frame
    // ---
    // --- This involves building a bone frame for each bone
    // --- frame in the existing data using the computed
    // --- character transforms
    // -------------------------------------------------------
    let mut final_pose_frame = BonePoseFrame::default();
    let inner_character_frame = final_pose_frame.inner_mut();

    for (path, bone_id) in data.inner_ref().paths.iter() {
        let local_bone_frame = &data.inner_ref().bones[*bone_id];
        let character_transform_frame = bone_transforms.get(path).unwrap();
        let character_translation_frame = character_transform_frame.map(|t| t.translation);
        let character_rotation_frame = character_transform_frame.map(|t| t.rotation);
        let character_scale_frame = character_transform_frame.map(|t| t.scale);

        let character_bone_frame = BoneFrame {
            rotation: Some(character_rotation_frame),
            translation: Some(character_translation_frame),
            scale: Some(character_scale_frame),
            weights: local_bone_frame.weights.clone(),
        };

        inner_character_frame.add_bone(character_bone_frame, path.clone());
    }
    // -------------------------------------------------------

    final_pose_frame
}

pub fn global_to_bone(data: &GlobalPoseFrame, ctx: PassContext) -> BonePoseFrame {
    let character_pose_frame = global_to_character(data, ctx.clone());

    character_to_bone(&character_pose_frame, ctx)
}

pub fn bone_to_global(data: &BonePoseFrame, ctx: PassContext) -> GlobalPoseFrame {
    let character_pose_frame = bone_to_character(data, ctx.clone());

    character_to_global(&character_pose_frame, ctx)
}