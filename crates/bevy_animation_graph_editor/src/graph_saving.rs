use crate::{ui::UiState, GraphHandles};
use bevy::{
    asset::io::{file::FileAssetReader, AssetReader, AssetSourceId},
    prelude::*,
};
use bevy_animation_graph::core::animation_graph::{serial::AnimationGraphSerial, AnimationGraph};
use std::path::PathBuf;

#[derive(Event)]
pub struct SaveGraph {
    pub graph: AssetId<AnimationGraph>,
    pub virtual_path: PathBuf,
}

pub fn save_graph_system(
    mut evr_save_graph: EventReader<SaveGraph>,
    asset_server: Res<AssetServer>,
    graph_assets: Res<Assets<AnimationGraph>>,
    mut ui_state: ResMut<UiState>,
    mut graph_handles: ResMut<GraphHandles>,
) {
    for ev in evr_save_graph.read() {
        let graph = graph_assets.get(ev.graph).unwrap();
        let graph_serial = AnimationGraphSerial::from(graph);
        let source = asset_server.get_source(AssetSourceId::Default).unwrap();
        let reader = source.reader();
        // HACK: Ideally we would not be doing this, but using bevy's dyanmic asset
        // saving API. However, this does not exist yet, so we're force to find the
        // file path via "other means".
        // unsafe downcast reader to FileAssetReader, since as_any is not implemented
        // pray that nothing goes wrong
        // TODO: An alternative could perhaps be to use the value in the Cli resource?
        let reader = unsafe { &*((reader as *const dyn AssetReader) as *const FileAssetReader) };
        let mut final_path = reader.root_path().clone();
        final_path.push(&ev.virtual_path);
        info!("Saving graph with id {:?} to {:?}", ev.graph, final_path);
        ron::ser::to_writer_pretty(
            std::fs::File::create(final_path).unwrap(),
            &graph_serial,
            ron::ser::PrettyConfig::default(),
        )
        .unwrap();

        // If we just saved a newly created graph, unload the in-memory asset from the
        // editor selection.
        // Also delete the temporary asset
        if asset_server.get_path(ev.graph).is_none() {
            ui_state.selection.graph_editor = None;
            graph_handles.unsaved.retain(|h| h.id() != ev.graph);
        }
    }
}
