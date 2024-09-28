mod two_bone;

pub use two_bone::*;

use bevy::app::App;

pub(super) fn register_types(app: &mut App) {
    app.register_type::<TwoBone>();
}
