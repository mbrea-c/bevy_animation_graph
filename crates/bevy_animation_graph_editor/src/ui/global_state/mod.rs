use bevy::ecs::{component::Component, entity::Entity, query::With, world::World};

use crate::ui::global_state::{
    active_fsm::{ActiveFsm, SetActiveFsm},
    active_fsm_state::SetActiveFsmState,
    active_fsm_transition::SetActiveFsmTransition,
    active_graph::ActiveGraph,
    active_graph_node::{ActiveGraphNode, SetActiveGraphNode},
    active_scene::{ActiveScene, SetActiveScene},
    inspector_selection::{InspectorSelection, SetInspectorSelection},
};

pub mod active_fsm;
pub mod active_fsm_state;
pub mod active_fsm_transition;
pub mod active_graph;
pub mod active_graph_node;
pub mod active_scene;
pub mod inspector_selection;

#[derive(Component)]
pub struct GlobalState;

impl GlobalState {
    pub fn init(world: &mut World) {
        let entity = world
            .spawn((
                GlobalState,
                ActiveScene::default(),
                ActiveFsm::default(),
                ActiveGraphNode::default(),
                InspectorSelection::default(),
            ))
            .id();

        ActiveGraph::register(world, entity);

        world.add_observer(SetActiveScene::observe);
        world.add_observer(SetActiveFsm::observe);
        world.add_observer(SetActiveGraphNode::observe);
        world.add_observer(SetInspectorSelection::observe);
        world.add_observer(SetActiveFsmState::observe);
        world.add_observer(SetActiveFsmTransition::observe);
    }
}

pub trait RegisterGlobalState {
    fn register(world: &mut World, global_state_entity: Entity);
}

pub fn get_global_state<T: Component>(world: &World) -> Option<&T> {
    let mut query_state = world.try_query_filtered::<&T, With<GlobalState>>()?;
    query_state.single(world).ok()
}
