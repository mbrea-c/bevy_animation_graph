use std::{
    fs, io,
    path::{Path, PathBuf},
};

use bevy::{asset::LoadedUntypedAsset, platform::collections::HashSet, prelude::*};

use crate::Cli;

pub struct ScannerPlugin;
impl Plugin for ScannerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(PersistedAssetHandles {
            loaded_paths: HashSet::default(),
        })
        .add_observer(asset_reload)
        .add_systems(Startup, core_setup);
    }
}

/// Keeps a handle to the folder so that it does not get unloaded
#[derive(Resource)]
pub struct PersistedAssetHandles {
    #[allow(dead_code)]
    pub loaded_paths: HashSet<Handle<LoadedUntypedAsset>>,
}

#[derive(Event)]
pub struct RescanAssets;

pub fn core_setup(mut gizmo_config: ResMut<GizmoConfigStore>, mut commands: Commands) {
    commands.trigger(RescanAssets);

    let config = gizmo_config.config_mut::<DefaultGizmoConfigGroup>().0;
    config.depth_bias = -1.;
}

pub fn asset_reload(
    _: On<RescanAssets>,
    asset_server: Res<AssetServer>,
    mut persisted_asset_handles: ResMut<PersistedAssetHandles>,
    cli: Res<Cli>,
) {
    visit_dirs(&cli.asset_source, &mut |path| {
        let relative_path = path.strip_prefix(&cli.asset_source).unwrap().to_owned();
        let loaded = asset_server.load_untyped(relative_path);
        persisted_asset_handles.loaded_paths.insert(loaded);
    })
    .unwrap_or_else(|err| {
        panic!(
            "Failed to load asset path {:?}: {:?}",
            cli.asset_source, err
        )
    });
}

// one possible implementation of walking a directory only visiting files
// taken from https://doc.rust-lang.org/nightly/std/fs/fn.read_dir.html#examples
fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(PathBuf)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                info!("Loading {path:?}");
                cb(path);
            }
        }
    }
    Ok(())
}
