use bevy::{
    asset::{Assets, Handle},
    ecs::{
        system::{In, ResMut},
        world::World,
    },
};
use bevy_animation_graph::core::ragdoll::definition::{Body, Collider, Joint, Ragdoll};

use crate::ui::actions::{DynamicAction, run_handler, saving::DirtyAssets};

pub struct EditRagdollBody {
    pub ragdoll: Handle<Ragdoll>,
    pub body: Body,
}

impl DynamicAction for EditRagdollBody {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not edit body")(Self::system, *self)
    }
}

impl EditRagdollBody {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        if let Some(body) = ragdoll.get_body_mut(input.body.id) {
            *body = input.body.clone()
        }
    }
}

pub struct CreateRagdollBody {
    pub ragdoll: Handle<Ragdoll>,
    pub body: Body,
}

impl DynamicAction for CreateRagdollBody {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not create body")(Self::system, *self)
    }
}

impl CreateRagdollBody {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        ragdoll.add_body(input.body);
    }
}

pub struct EditRagdollCollider {
    pub ragdoll: Handle<Ragdoll>,
    pub collider: Collider,
}

impl DynamicAction for EditRagdollCollider {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not edit collider")(Self::system, *self)
    }
}

impl EditRagdollCollider {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        if let Some(collider) = ragdoll.get_collider_mut(input.collider.id) {
            *collider = input.collider.clone()
        }
    }
}

pub struct EditRagdollJoint {
    pub ragdoll: Handle<Ragdoll>,
    pub joint: Joint,
}

impl DynamicAction for EditRagdollJoint {
    fn handle(self: Box<Self>, world: &mut World) {
        run_handler(world, "Could not edit joint")(Self::system, *self)
    }
}

impl EditRagdollJoint {
    pub fn system(
        In(input): In<Self>,
        mut ragdoll_assets: ResMut<Assets<Ragdoll>>,
        mut dirty_assets: ResMut<DirtyAssets>,
    ) {
        let Some(ragdoll) = ragdoll_assets.get_mut(&input.ragdoll) else {
            return;
        };

        dirty_assets.add(input.ragdoll);

        if let Some(joint) = ragdoll.get_joint_mut(input.joint.id) {
            *joint = input.joint.clone()
        }
    }
}
