mod dummy;
mod math;

pub mod entity_path;
pub mod graph;
// pub mod space_conversion;
pub mod bool;
pub mod f32;
pub mod ik;
pub mod pose;
pub mod quat;
pub mod vec3;

pub use dummy::*;

use bevy::app::App;

pub(crate) fn register_types(app: &mut App) {
    app.register_type::<Dummy>();

    entity_path::register_types(app);
    graph::register_types(app);
    bool::register_types(app);
    f32::register_types(app);
    ik::register_types(app);
    pose::register_types(app);
    quat::register_types(app);
    vec3::register_types(app);
}
