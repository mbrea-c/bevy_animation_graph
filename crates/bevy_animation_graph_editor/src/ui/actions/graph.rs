use bevy::{
    asset::{AssetId, Handle},
    math::Vec2,
};
use bevy_animation_graph::{
    core::animation_graph::{NodeId, SourcePin, TargetPin},
    prelude::{AnimationGraph, AnimationNode},
};

pub enum GraphAction {
    CreateLink(CreateLink),
    RemoveLink(RemoveLink),
    MoveNode(MoveNode),
    MoveInput(MoveInput),
    MoveOutput(MoveOutput),
    RenameNode(RenameNode),
    CreateNode(CreateNode),
    RemoveNode(RemoveNode),
    Noop,
    GraphValidate,
}

pub struct CreateLink {
    graph: Handle<AnimationGraph>,
    source: SourcePin,
    target: TargetPin,
}

pub struct RemoveLink {
    graph: Handle<AnimationGraph>,
    target: TargetPin,
}

pub struct MoveNode {
    graph: Handle<AnimationGraph>,
    node: NodeId,
    new_pos: Vec2,
}

pub struct MoveInput {
    graph: Handle<AnimationGraph>,
    new_pos: Vec2,
}

pub struct MoveOutput {
    graph: Handle<AnimationGraph>,
    new_pos: Vec2,
}

pub struct RenameNode {
    graph: Handle<AnimationGraph>,
    node: NodeId,
    new_name: String,
}

pub struct CreateNode {
    graph: Handle<AnimationGraph>,
    node: AnimationNode,
}

pub struct RemoveNode {
    graph: Handle<AnimationGraph>,
    node: NodeId,
}
