use bevy::{
    asset::{Assets, Handle},
    ecs::{
        system::{In, ResMut},
        world::World,
    },
};
use bevy_animation_graph::{
    core::{
        colliders::core::{ColliderConfig, SkeletonColliderId, SkeletonColliders},
        id::BoneId,
    },
    prelude::config::SymmetryConfig,
};

use crate::ui::actions::{ActionContext, DynamicAction, run_handler, saving::DirtyAssets};

pub struct CreateOrEditCollider {
    pub colliders: Handle<SkeletonColliders>,
    pub config: ColliderConfig,
}

impl DynamicAction for CreateOrEditCollider {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
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
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
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

pub struct UpdateSymmetryConfig {
    pub colliders: Handle<SkeletonColliders>,
    pub symmetry: SymmetryConfig,
}

impl DynamicAction for UpdateSymmetryConfig {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not update symmetry config")(Self::system, *self)
    }
}

impl UpdateSymmetryConfig {
    pub fn system(
        In(input): In<Self>,
        mut skeleton_collider_assets: ResMut<Assets<SkeletonColliders>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(skeleton_colliders) = skeleton_collider_assets.get_mut(&input.colliders) else {
            return;
        };

        dirty_assets.add(input.colliders);
        skeleton_colliders.symmetry = input.symmetry;
    }
}

pub struct UpdateSymmetryEnabled {
    pub colliders: Handle<SkeletonColliders>,
    pub symmetry_enabled: bool,
}

impl DynamicAction for UpdateSymmetryEnabled {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not update symmetry config")(Self::system, *self)
    }
}

impl UpdateSymmetryEnabled {
    pub fn system(
        In(input): In<Self>,
        mut skeleton_collider_assets: ResMut<Assets<SkeletonColliders>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(skeleton_colliders) = skeleton_collider_assets.get_mut(&input.colliders) else {
            return;
        };

        dirty_assets.add(input.colliders);
        skeleton_colliders.symmetry_enabled = input.symmetry_enabled;
    }
}

pub struct UpdateDefaultLayers {
    pub colliders: Handle<SkeletonColliders>,
    pub layer_membership: u32,
    pub layer_filter: u32,
}

impl DynamicAction for UpdateDefaultLayers {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not update default physics layers config")(Self::system, *self)
    }
}

impl UpdateDefaultLayers {
    pub fn system(
        In(input): In<Self>,
        mut skeleton_collider_assets: ResMut<Assets<SkeletonColliders>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(skeleton_colliders) = skeleton_collider_assets.get_mut(&input.colliders) else {
            return;
        };

        dirty_assets.add(input.colliders);
        skeleton_colliders.default_layer_membership = input.layer_membership;
        skeleton_colliders.default_layer_filter = input.layer_filter;
    }
}

pub struct UpdateSuffixes {
    pub colliders: Handle<SkeletonColliders>,
    pub suffix: String,
    pub mirror_suffix: String,
}

impl DynamicAction for UpdateSuffixes {
    fn handle(self: Box<Self>, world: &mut World, _: &mut ActionContext) {
        run_handler(world, "Could not update suffixes config")(Self::system, *self)
    }
}

impl UpdateSuffixes {
    pub fn system(
        In(input): In<Self>,
        mut skeleton_collider_assets: ResMut<Assets<SkeletonColliders>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(skeleton_colliders) = skeleton_collider_assets.get_mut(&input.colliders) else {
            return;
        };

        dirty_assets.add(input.colliders);
        skeleton_colliders.suffix = input.suffix;
        skeleton_colliders.mirror_suffix = input.mirror_suffix;
    }
}
