use bevy::{
    asset::{Assets, Handle},
    ecs::{
        system::{In, ResMut},
        world::World,
    },
};
use bevy_animation_graph::core::{
    colliders::core::{ColliderConfig, SkeletonColliderId, SkeletonColliders},
    id::BoneId,
};

use crate::ui::actions::{DynamicAction, run_handler, saving::DirtyAssets};

pub struct CreateOrEditCollider {
    pub colliders: Handle<SkeletonColliders>,
    pub config: ColliderConfig,
}

impl DynamicAction for CreateOrEditCollider {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not create collider")(Self::system, *self)
    }
}

impl CreateOrEditCollider {
    pub fn system(
        In(input): In<Self>,
        mut skeleton_collider_assets: ResMut<Assets<SkeletonColliders>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(skeleton_colliders) = skeleton_collider_assets.get_mut(&input.colliders) else {
            return;
        };

        dirty_assets.add(input.colliders);

        if let Some(cfg) = skeleton_colliders
            .get_colliders_mut(input.config.attached_to)
            .and_then(|colls| colls.iter_mut().find(|cfg| cfg.id == input.config.id))
        {
            *cfg = input.config;
        } else {
            skeleton_colliders.add_collider(input.config);
        }
    }
}

pub struct DeleteCollider {
    pub colliders: Handle<SkeletonColliders>,
    pub bone_id: BoneId,
    pub collider_id: SkeletonColliderId,
}

impl DynamicAction for DeleteCollider {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not create collider")(Self::system, *self)
    }
}

impl DeleteCollider {
    pub fn system(
        In(input): In<Self>,
        mut skeleton_collider_assets: ResMut<Assets<SkeletonColliders>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(skeleton_colliders) = skeleton_collider_assets.get_mut(&input.colliders) else {
            return;
        };

        dirty_assets.add(input.colliders);

        skeleton_colliders.delete_collider(input.bone_id, input.collider_id);
    }
}
