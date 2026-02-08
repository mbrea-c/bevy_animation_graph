use bevy::{
    asset::UntypedAssetId,
    ecs::{
        component::Component, entity::Entity, event::Event, observer::On, query::With,
        system::Single, world::World,
    },
    platform::collections::HashMap,
};
use bevy_animation_graph::core::context::{
    graph_context::GraphState, graph_context_arena::GraphContextId,
};

use crate::ui::{
    native_windows::OwnedQueue,
    state_management::global::{GlobalState, RegisterStateComponent, get_global_state},
    utils,
};

#[derive(Debug, Component, Clone, Default)]
pub struct ActiveContexts {
    pub by_asset: HashMap<UntypedAssetId, (Entity, GraphContextId)>,
}

impl RegisterStateComponent for ActiveContexts {
    fn register(world: &mut World, global_state_entity: Entity) {
        world
            .entity_mut(global_state_entity)
            .insert(ActiveContexts::default());

        world.add_observer(SetActiveContext::observe);
    }
}

#[derive(Event)]
pub struct SetActiveContext {
    pub asset_id: UntypedAssetId,
    pub entity: Entity,
    pub id: GraphContextId,
}

impl SetActiveContext {
    pub fn observe(
        event: On<SetActiveContext>,
        mut global_state: Single<&mut ActiveContexts, With<GlobalState>>,
    ) {
        let event = event.event();
        global_state
            .by_asset
            .insert(event.asset_id, (event.entity, event.id));
    }
}

pub fn ensure_active_context_selected_if_available(
    world: &World,
    id: UntypedAssetId,
    filter: impl Fn(&GraphState) -> bool,
    queue: &mut OwnedQueue,
) {
    if get_global_state::<ActiveContexts>(world).is_some_and(|c| c.by_asset.contains_key(&id)) {
        return;
    }

    if let Some((entity, graph_context_id)) = list_graph_contexts(world, filter).into_iter().next()
    {
        queue.trigger(SetActiveContext {
            asset_id: id,
            entity,
            id: graph_context_id,
        });
    }
}

fn list_graph_contexts(
    world: &World,
    filter: impl Fn(&GraphState) -> bool,
) -> Vec<(Entity, GraphContextId)> {
    let players = utils::iter_animation_graph_players(world);
    players
        .iter()
        .filter_map(|(entity, player)| Some((entity, player.get_context_arena()?)))
        .flat_map(|(entity, arena)| {
            arena
                .iter_context_ids()
                .filter(|id| {
                    let context = arena.get_context(*id).unwrap();
                    filter(context)
                })
                .map(|id| (*entity, id))
        })
        .collect()
}
