mod fsm;
mod graph;
mod send_event;

pub use fsm::*;
pub use graph::*;
pub use send_event::*;

use bevy::app::App;

pub(super) fn register_types(app: &mut App) {
    app.register_type::<Fsm>()
        .register_type::<Graph>()
        .register_type::<SendEvent>();
}
