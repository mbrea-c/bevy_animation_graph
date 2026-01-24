use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::Event,
    observer::On,
    system::{Commands, Res, ResMut},
    world::World,
};
use bevy_animation_graph::core::ragdoll::bone_mapping_loader::RagdollBoneMapSerial;

use crate::{
    Cli,
    scanner::RescanAssets,
    ui::{
        UiState,
        core::EguiWindow,
        native_windows::{
            NativeEditorWindow, asset_creation::ragdoll_bone_map::CreateRagdollBoneMapWindow,
        },
        state_management::global::RegisterStateComponent,
    },
};

#[derive(Debug, Component, Default, Clone)]
pub struct RagdollBoneMapManager;

impl RegisterStateComponent for RagdollBoneMapManager {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(RequestCreateRagdollBoneMap::observe);
        world.add_observer(CreateRagdollBoneMap::observe);
    }
}

/// Will open a "create FSM" window popup
#[derive(Event)]
pub struct RequestCreateRagdollBoneMap;

impl RequestCreateRagdollBoneMap {
    pub fn observe(
        _: On<RequestCreateRagdollBoneMap>,
        mut commands: Commands,
        ui_state: ResMut<UiState>,
    ) {
        if let Some(active_view_idx) = ui_state.active_view {
            let view = &ui_state.views[active_view_idx];
            let win = NativeEditorWindow::create_cmd(
                &mut commands,
                view.entity,
                CreateRagdollBoneMapWindow,
            );

            let UiState { views, .. } = ui_state.into_inner();

            views[active_view_idx]
                .dock_state
                .add_window(vec![EguiWindow::EntityWindow(win)]);
        }
    }
}

#[derive(Event)]
pub struct CreateRagdollBoneMap {
    pub virtual_path: String,
    pub ragdoll_bone_map: RagdollBoneMapSerial,
}

impl CreateRagdollBoneMap {
    pub fn observe(event: On<CreateRagdollBoneMap>, cli: Res<Cli>, mut commands: Commands) {
        let mut final_path = cli.asset_source.clone();
        final_path.push(&event.virtual_path);
        bevy::log::info!("Creating ragdoll bone map at {:?}", final_path);
        ron::Options::default()
            .to_io_writer_pretty(
                std::fs::File::create(final_path).unwrap(),
                &event.ragdoll_bone_map,
                ron::ser::PrettyConfig::default(),
            )
            .unwrap();

        commands.trigger(RescanAssets);
    }
}
