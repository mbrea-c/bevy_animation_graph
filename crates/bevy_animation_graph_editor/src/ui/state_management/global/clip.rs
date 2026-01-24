use bevy::ecs::{
    component::Component,
    entity::Entity,
    event::Event,
    observer::On,
    system::{Commands, Res, ResMut},
    world::World,
};
use bevy_animation_graph::core::animation_clip::loader::GraphClipSerial;

use crate::{
    Cli,
    scanner::RescanAssets,
    ui::{
        UiState,
        core::EguiWindow,
        native_windows::{NativeEditorWindow, asset_creation::anim::CreateAnimWindow},
        state_management::global::RegisterStateComponent,
    },
};

#[derive(Debug, Component, Default, Clone)]
pub struct ClipManager;

impl RegisterStateComponent for ClipManager {
    fn register(world: &mut World, _global_state_entity: Entity) {
        world.add_observer(RequestCreateClip::observe);
        world.add_observer(CreateClip::observe);
    }
}

/// Will open a "create FSM" window popup
#[derive(Event)]
pub struct RequestCreateClip;

impl RequestCreateClip {
    pub fn observe(_: On<RequestCreateClip>, mut commands: Commands, ui_state: ResMut<UiState>) {
        if let Some(active_view_idx) = ui_state.active_view {
            let view = &ui_state.views[active_view_idx];
            let win = NativeEditorWindow::create_cmd(&mut commands, view.entity, CreateAnimWindow);

            let UiState { views, .. } = ui_state.into_inner();

            views[active_view_idx]
                .dock_state
                .add_window(vec![EguiWindow::EntityWindow(win)]);
        }
    }
}

#[derive(Event)]
pub struct CreateClip {
    pub virtual_path: String,
    pub clip_serial: GraphClipSerial,
}

impl CreateClip {
    pub fn observe(event: On<CreateClip>, cli: Res<Cli>, mut commands: Commands) {
        let mut final_path = cli.asset_source.clone();
        final_path.push(&event.virtual_path);
        bevy::log::info!("Creating animation clip metadata at {:?}", final_path);
        ron::Options::default()
            .to_io_writer_pretty(
                std::fs::File::create(final_path).unwrap(),
                &event.clip_serial,
                ron::ser::PrettyConfig::default(),
            )
            .unwrap();

        commands.trigger(RescanAssets);
    }
}
