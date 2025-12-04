use bevy::{
    asset::UntypedAssetId,
    ecs::{
        component::Component, entity::Entity, event::Event, observer::On, query::With,
        system::Single, world::World,
    },
    platform::collections::HashMap,
};
use bevy_animation_graph::core::context::graph_context_arena::GraphContextId;

use crate::ui::global_state::{GlobalState, RegisterStateComponent};

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
