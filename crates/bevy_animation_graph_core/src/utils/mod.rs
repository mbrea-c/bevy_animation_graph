pub mod delaunay;
pub mod geometry;
pub mod loading;

use bevy::asset::AssetPath;
use std::path::PathBuf;

/// Normalize an [`AssetPath`] to use forward slashes, ensuring portability across platforms.
///
/// On Windows, `std::path::Path` uses backslashes as separators. This function converts them to
/// forward slashes so that serialized asset paths can be loaded on any platform.
pub fn normalize_asset_path(path: AssetPath<'static>) -> AssetPath<'static> {
    let normalized =
        AssetPath::from_path_buf(PathBuf::from(path.path().to_string_lossy().replace('\\', "/")));
    match path.label() {
        Some(label) => normalized.with_label(label.to_string()).into_owned(),
        None => normalized.into_owned(),
    }
}
