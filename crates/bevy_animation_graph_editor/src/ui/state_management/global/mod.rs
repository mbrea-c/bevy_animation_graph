pub mod active_fsm;
pub mod active_fsm_state;
pub mod active_fsm_transition;
pub mod active_graph;
pub mod active_graph_context;
pub mod active_graph_node;
pub mod active_ragdoll;
pub mod active_scene;
pub mod active_skeleton;
pub mod fsm;
pub mod inspector_selection;

use std::marker::PhantomData;

use bevy::ecs::{
    component::{Component, Mutable},
    entity::Entity,
    error::BevyError,
    event::{EntityEvent, Event},
    observer::On,
    query::With,
    system::{Commands, Query, Single},
    world::World,
};

use crate::ui::state_management::global::{
    active_fsm::ActiveFsm, active_fsm_state::ActiveFsmState,
    active_fsm_transition::ActiveFsmTransition, active_graph::ActiveGraph,
    active_graph_context::ActiveContexts, active_graph_node::ActiveGraphNode,
    active_ragdoll::ActiveRagdoll, active_scene::ActiveScene, active_skeleton::ActiveSkeleton,
    fsm::FsmManager, inspector_selection::InspectorSelection,
};

#[derive(Component)]
pub struct GlobalState;

impl GlobalState {
    pub fn init(world: &mut World) {
        let entity = world.spawn(GlobalState).id();

        ActiveContexts::register(world, entity);
        ActiveFsm::register(world, entity);
        ActiveFsmState::register(world, entity);
        ActiveFsmTransition::register(world, entity);
        ActiveGraph::register(world, entity);
        ActiveGraphNode::register(world, entity);
        ActiveRagdoll::register(world, entity);
        ActiveScene::register(world, entity);
        ActiveSkeleton::register(world, entity);
        InspectorSelection::register(world, entity);
        FsmManager::register(world, entity);

        world.add_observer(CloseWindow::observe);
    }
}

pub trait RegisterStateComponent: Component {
    fn register(world: &mut World, state_entity: Entity);
}

pub fn register_if_missing<T: RegisterStateComponent>(world: &mut World, state_entity: Entity) {
    if !world.entity(state_entity).contains::<T>() {
        T::register(world, state_entity);
    }
}

pub fn get_global_state<T: Component>(world: &World) -> Option<&T> {
    let mut query_state = world.try_query_filtered::<&T, With<GlobalState>>()?;
    query_state.single(world).ok()
}

// For convenience, since it's a common pattern
pub trait SetOrInsertEvent: Event {
    type Target: Component;

    fn get_component(&self) -> Self::Target;
}

fn observe_set_or_insert_event<T, E>(
    new_state: On<E>,
    global_state: Single<(Entity, Option<&mut T>), With<GlobalState>>,
    mut commands: Commands,
) where
    T: Component<Mutability = Mutable>,
    E: SetOrInsertEvent<Target = T>,
{
    let (entity, old_state) = global_state.into_inner();

    if let Some(mut old_state) = old_state {
        *old_state = new_state.event().get_component();
    } else {
        commands
            .entity(entity)
            .insert(new_state.event().get_component());
    }
}

#[derive(Default, Event)]
pub struct ClearGlobalState<T>(PhantomData<T>);

pub fn observe_clear_global_state<T: Component>(
    _: On<ClearGlobalState<T>>,
    global_state: Query<Entity, With<GlobalState>>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let global_state_entity = global_state.single()?;
    commands.entity(global_state_entity).remove::<T>();
    Ok(())
}

// Windowing stuff (unintuitively this needs to be registered globally so it does belong here)

#[derive(EntityEvent)]
pub struct CloseWindow(pub Entity);

impl Default for CloseWindow {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

impl CloseWindow {
    pub fn observe(event: On<CloseWindow>, mut commands: Commands) {
        commands.entity(event.0).despawn();
    }
}
