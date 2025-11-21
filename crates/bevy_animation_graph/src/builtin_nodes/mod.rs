use bevy::app::{App, Plugin};

use crate::builtin_nodes::{
    blend_node::BlendNode,
    blend_space_node::BlendSpaceNode,
    chain_node::ChainNode,
    clip_node::ClipNode,
    const_ragdoll_config::ConstRagdollConfig,
    dummy_node::DummyNode,
    event_queue::fire_event::FireEventNode,
    f32::{
        abs_f32::AbsF32, add_f32::AddF32, clamp_f32::ClampF32, compare_f32::CompareF32,
        div_f32::DivF32, mul_f32::MulF32, sub_f32::SubF32,
    },
    flip_lr_node::FlipLRNode,
    fsm_node::FSMNode,
    graph_node::GraphNode,
    loop_node::LoopNode,
    padding::PaddingNode,
    rotation_node::RotationNode,
    speed_node::SpeedNode,
    twoboneik_node::TwoBoneIKNode,
    vec3::rotation_arc::RotationArcNode,
};

pub mod blend_node;
pub mod blend_space_node;
pub mod bool;
pub mod chain_node;
pub mod clip_node;
pub mod const_entity_path;
pub mod const_ragdoll_config;
pub mod dummy_node;
pub mod event_markup_node;
pub mod event_queue;
pub mod f32;
pub mod flip_lr_node;
pub mod fsm_node;
pub mod graph_node;
pub mod loop_node;
pub mod padding;
pub mod quat;
pub mod rotation_node;
pub mod speed_node;
pub mod twoboneik_node;
pub mod vec3;

pub struct BuiltinNodesPlugin;

impl Plugin for BuiltinNodesPlugin {
    fn build(&self, app: &mut App) {
        self.register_nodes(app);
        self.register_other(app);
    }
}

impl BuiltinNodesPlugin {
    /// Registers built-in animation node implementations
    fn register_nodes(&self, app: &mut App) {
        app //
            .register_type::<ClipNode>()
            .register_type::<DummyNode>()
            .register_type::<ChainNode>()
            .register_type::<BlendNode>()
            .register_type::<BlendSpaceNode>()
            .register_type::<FlipLRNode>()
            .register_type::<GraphNode>()
            .register_type::<LoopNode>()
            .register_type::<PaddingNode>()
            .register_type::<RotationNode>()
            .register_type::<SpeedNode>()
            .register_type::<FSMNode>()
            .register_type::<TwoBoneIKNode>()
            // f32
            .register_type::<AbsF32>()
            .register_type::<AddF32>()
            .register_type::<ClampF32>()
            .register_type::<DivF32>()
            .register_type::<MulF32>()
            .register_type::<SubF32>()
            .register_type::<CompareF32>()
            // quat
            .register_type::<RotationArcNode>()
            // event queue
            .register_type::<FireEventNode>()
            // ragdoll
            .register_type::<ConstRagdollConfig>();
    }

    fn register_other(&self, app: &mut App) {}
}
