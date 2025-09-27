use bevy::{
    asset::{Assets, Handle},
    ecs::{
        system::{In, ResMut},
        world::World,
    },
};
use bevy_animation_graph::core::ragdoll::definition::{Body, Ragdoll};

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
