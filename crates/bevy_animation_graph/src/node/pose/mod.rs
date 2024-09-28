mod blend;
mod chain;
mod change_speed;
mod clip;
mod flip_left_right;
mod padding;
mod repeat;
mod rotate;

pub use blend::*;
pub use chain::*;
pub use change_speed::*;
pub use clip::*;
pub use flip_left_right::*;
pub use padding::*;
pub use repeat::*;
pub use rotate::*;

use bevy::app::App;

pub(super) fn register_types(app: &mut App) {
    app.register_type::<Clip>()
        .register_type::<Chain>()
        .register_type::<Blend>()
        .register_type::<FlipLeftRight>()
        .register_type::<Repeat>()
        .register_type::<Padding>()
        .register_type::<Rotate>()
        .register_type::<ChangeSpeed>();
}
