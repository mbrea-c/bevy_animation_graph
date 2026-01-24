use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::Event,
    observer::On,
    system::{Commands, Res, ResMut},
    world::World,
};
use bevy_animation_graph::core::skeleton::serial::SkeletonSerial;

use crate::{
    Cli,
    scanner::RescanAssets,
    ui::{
        UiState,
        core::EguiWindow,
        native_windows::{NativeEditorWindow, asset_creation::skeleton::CreateSkeletonWindow},
        state_management::global::RegisterStateComponent,
    },
};

#[derive(Debug, Component, Default, Clone)]
pub struct SkeletonManager;

impl RegisterStateComponent for SkeletonManager {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(RequestCreateSkeleton::observe);
        world.add_observer(CreateSkeleton::observe);
    }
}

/// Will open a "create FSM" window popup
#[derive(Event)]
pub struct RequestCreateSkeleton;

impl RequestCreateSkeleton {
    pub fn observe(
        _: On<RequestCreateSkeleton>,
        mut commands: Commands,
        ui_state: ResMut<UiState>,
    ) {
        if let Some(active_view_idx) = ui_state.active_view {
            let view = &ui_state.views[active_view_idx];
            let win =
                NativeEditorWindow::create_cmd(&mut commands, view.entity, CreateSkeletonWindow);

            let UiState { views, .. } = ui_state.into_inner();

            views[active_view_idx]
                .dock_state
                .add_window(vec![EguiWindow::EntityWindow(win)]);
        }
    }
}

#[derive(Event)]
pub struct CreateSkeleton {
    pub virtual_path: String,
    pub skeleton: SkeletonSerial,
}

impl CreateSkeleton {
    pub fn observe(event: On<CreateSkeleton>, cli: Res<Cli>, mut commands: Commands) {
        let mut final_path = cli.asset_source.clone();
        final_path.push(&event.virtual_path);
        bevy::log::info!("Creating skeleton at {:?}", final_path);
        ron::Options::default()
            .to_io_writer_pretty(
                std::fs::File::create(final_path).unwrap(),
                &event.skeleton,
                ron::ser::PrettyConfig::default(),
            )
            .unwrap();

        commands.trigger(RescanAssets);
    }
}
