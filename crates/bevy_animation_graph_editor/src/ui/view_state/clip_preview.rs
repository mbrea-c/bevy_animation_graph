use bevy::{
    asset::Handle,
    ecs::{
        component::Component, entity::Entity, event::Event, observer::Trigger, system::Query,
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

#[derive(Event)]
pub struct SetElapsedTime(pub f32);

impl SetElapsedTime {
    pub fn observe(event: Trigger<SetElapsedTime>, mut query: Query<&mut ClipPreviewViewState>) {
        if let Ok(mut state) = query.get_mut(event.target()) {
            state.elapsed_time = event.event().0;
        }
    }
}

#[derive(Event)]
pub struct SetOrder(pub ClipPreviewTimingOrder);

impl SetOrder {
    pub fn observe(event: Trigger<SetOrder>, mut query: Query<&mut ClipPreviewViewState>) {
        if let Ok(mut state) = query.get_mut(event.target()) {
            state.order = Some(OrderStatus {
                uuid: Uuid::new_v4(),
                order: event.event().0.clone(),
                applied_by: HashSet::new(),
            });
        }
    }
}

#[derive(Event)]
pub struct OrderApplied {
    pub uuid: Uuid,
    pub window: Entity,
}

impl OrderApplied {
    pub fn observe(event: Trigger<OrderApplied>, mut query: Query<&mut ClipPreviewViewState>) {
        if let Ok(mut state) = query.get_mut(event.target())
            && let Some(order) = &mut state.order
            && event.event().uuid == order.uuid
        {
            order.applied_by.insert(event.event().window);
        }
    }
}

#[derive(Event)]
pub struct SetTargetTracks {
    pub tracks: Option<TargetTracks>,
}

impl SetTargetTracks {
    pub fn observe(event: Trigger<SetTargetTracks>, mut query: Query<&mut ClipPreviewViewState>) {
        if let Ok(mut state) = query.get_mut(event.target()) {
            state.target_tracks = event.event().tracks.clone();
        }
    }
}

#[derive(Event)]
pub struct SetClipPreviewBaseScene(pub Handle<AnimatedScene>);

impl SetClipPreviewBaseScene {
    pub fn observe(
        event: Trigger<SetClipPreviewBaseScene>,
        mut query: Query<&mut ClipPreviewViewState>,
    ) {
        if let Ok(mut state) = query.get_mut(event.target()) {
            state.base_scene = Some(event.event().0.clone());
        }
    }
}
