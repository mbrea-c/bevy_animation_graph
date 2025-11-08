use bevy::{
    asset::Handle,
    ecs::{
        component::Component, entity::Entity, event::EntityEvent, observer::On, system::Query,
        world::World,
    },
    platform::collections::HashSet,
};
use bevy_animation_graph::prelude::AnimatedScene;
use uuid::Uuid;

use crate::ui::{
    global_state::RegisterStateComponent, native_windows::event_track_editor::TargetTracks,
};

#[derive(Debug, Clone)]
pub enum ClipPreviewTimingOrder {
    Seek { time: f32 },
}

#[derive(Component, Default)]
pub struct ClipPreviewViewState {
    elapsed_time: f32,
    order: Option<OrderStatus>,
    /// The track we're previewing
    target_tracks: Option<TargetTracks>,
    base_scene: Option<Handle<AnimatedScene>>,
}

impl ClipPreviewViewState {
    pub fn elapsed_time(&self) -> f32 {
        self.elapsed_time
    }

    pub fn target_tracks(&self) -> Option<TargetTracks> {
        self.target_tracks.clone()
    }

    pub fn base_scene(&self) -> Option<Handle<AnimatedScene>> {
        self.base_scene.clone()
    }

    pub fn order_status(&self) -> Option<&OrderStatus> {
        self.order.as_ref()
    }
}

impl RegisterStateComponent for ClipPreviewViewState {
    fn register(world: &mut World, state_entity: Entity) {
        world
            .entity_mut(state_entity)
            .insert(ClipPreviewViewState::default())
            .observe(SetElapsedTime::observe)
            .observe(SetOrder::observe)
            .observe(OrderApplied::observe)
            .observe(SetTargetTracks::observe)
            .observe(SetClipPreviewBaseScene::observe);
    }
}

#[derive(Clone)]
pub struct OrderStatus {
    pub uuid: Uuid,
    pub order: ClipPreviewTimingOrder,
    /// Which windows have already applied this order
    pub applied_by: HashSet<Entity>,
}

#[derive(EntityEvent)]
pub struct SetElapsedTime {
    pub entity: Entity,
    pub time: f32,
}

impl SetElapsedTime {
    pub fn observe(event: On<SetElapsedTime>, mut query: Query<&mut ClipPreviewViewState>) {
        if let Ok(mut state) = query.get_mut(event.entity) {
            state.elapsed_time = event.time;
        }
    }
}

#[derive(EntityEvent)]
pub struct SetOrder {
    pub entity: Entity,
    pub order: ClipPreviewTimingOrder,
}

impl SetOrder {
    pub fn observe(event: On<SetOrder>, mut query: Query<&mut ClipPreviewViewState>) {
        if let Ok(mut state) = query.get_mut(event.entity) {
            state.order = Some(OrderStatus {
                uuid: Uuid::new_v4(),
                order: event.order.clone(),
                applied_by: HashSet::new(),
            });
        }
    }
}

#[derive(EntityEvent)]
pub struct OrderApplied {
    /// View this event is targeted to
    pub entity: Entity,
    pub uuid: Uuid,
    /// Window that has applied the order
    pub window: Entity,
}

impl OrderApplied {
    pub fn observe(event: On<OrderApplied>, mut query: Query<&mut ClipPreviewViewState>) {
        if let Ok(mut state) = query.get_mut(event.entity)
            && let Some(order) = &mut state.order
            && event.uuid == order.uuid
        {
            order.applied_by.insert(event.window);
        }
    }
}

#[derive(EntityEvent)]
pub struct SetTargetTracks {
    pub entity: Entity,
    pub tracks: Option<TargetTracks>,
}

impl SetTargetTracks {
    pub fn observe(event: On<SetTargetTracks>, mut query: Query<&mut ClipPreviewViewState>) {
        if let Ok(mut state) = query.get_mut(event.entity) {
            state.target_tracks = event.tracks.clone();
        }
    }
}

#[derive(EntityEvent)]
pub struct SetClipPreviewBaseScene {
    pub entity: Entity,
    pub scene: Handle<AnimatedScene>,
}

impl SetClipPreviewBaseScene {
    pub fn observe(
        event: On<SetClipPreviewBaseScene>,
        mut query: Query<&mut ClipPreviewViewState>,
    ) {
        if let Ok(mut state) = query.get_mut(event.entity) {
            state.base_scene = Some(event.scene.clone());
        }
    }
}
