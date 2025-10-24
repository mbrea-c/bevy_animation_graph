use bevy::ecs::{
    component::Component,
    entity::Entity,
    query::With,
    world::{EntityMut, World},
};

use crate::ui::global_state::active_scene::{ActiveScene, SetActiveScene};

pub mod active_scene;

#[derive(Component)]
pub struct GlobalState;

impl GlobalState {
    pub fn init(world: &mut World) {
        world.spawn((GlobalState, ActiveScene::default()));

        world.add_observer(SetActiveScene::observe);
    }
}

pub fn get_global_state<T: Component>(world: &World) -> Option<&T> {
    let mut query_state = world.try_query_filtered::<&T, With<GlobalState>>()?;
    query_state.single(world).ok()
}
