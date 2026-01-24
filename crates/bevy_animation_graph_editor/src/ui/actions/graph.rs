use std::fmt::Display;

use bevy::{
    asset::{AssetId, Assets, Handle},
    ecs::{
        system::{In, Res, ResMut, SystemParam},
        world::World,
    },
    log::{error, info, warn},
    math::Vec2,
    platform::collections::HashMap,
};
use bevy_animation_graph::core::{
    animation_graph::{AnimationGraph, Edge, GraphInputPin, NodeId, SourcePin, TargetPin},
    animation_node::{AnimationNode, dyn_node_like::DynNodeLike},
    context::spec_context::{GraphSpec, SpecResources},
    edge_data::DataValue,
    state_machine::high_level::StateMachine,
};

use super::saving::DirtyAssets;
use crate::graph_show::{GraphIndicesMap, make_graph_indices};

pub enum GraphAction {
    CreateLink(CreateLink),
    RemoveLink(RemoveLink),
    MoveNode(MoveNode),
    MoveInput(MoveInput),
    MoveOutput(MoveOutput),
    RenameNode(RenameNode),
    CreateNode(CreateNode),
    EditNode(EditNode),
    RemoveNode(RemoveNode),
    UpdateDefaultData(UpdateDefaultData),
    UpdateGraphSpec(UpdateGraphSpec),
    Noop,
    GenerateIndices(GenerateIndices),
}

pub struct CreateLink {
    pub graph: Handle<AnimationGraph>,
    pub source: SourcePin,
    pub target: TargetPin,
}

pub struct RemoveLink {
    pub graph: Handle<AnimationGraph>,
    pub target: TargetPin,
}

pub struct MoveNode {
    pub graph: Handle<AnimationGraph>,
    pub node: NodeId,
    pub new_pos: Vec2,
}

pub struct MoveInput {
    pub graph: Handle<AnimationGraph>,
    pub new_pos: Vec2,
}

pub struct MoveOutput {
    pub graph: Handle<AnimationGraph>,
    pub new_pos: Vec2,
}

pub struct RenameNode {
    pub graph: Handle<AnimationGraph>,
    pub node: NodeId,
    pub new_name: String,
}

pub struct CreateNode {
    pub graph: Handle<AnimationGraph>,
    pub node: AnimationNode,
}

pub struct EditNode {
    pub graph: Handle<AnimationGraph>,
    pub node: NodeId,
    pub new_inner: DynNodeLike,
}

pub struct RemoveNode {
    pub graph: Handle<AnimationGraph>,
    pub node: NodeId,
}

pub struct UpdateDefaultData {
    pub graph: Handle<AnimationGraph>,
    pub input_data: HashMap<GraphInputPin, DataValue>,
}

pub struct UpdateGraphSpec {
    pub graph: Handle<AnimationGraph>,
    pub new_spec: GraphSpec,
}

pub struct GenerateIndices {
    pub graph: AssetId<AnimationGraph>,
}

pub fn handle_graph_action(world: &mut World, action: GraphAction) {
    match action {
        GraphAction::CreateLink(action) => {
            let _ = world
                .run_system_cached_with(create_link_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::RemoveLink(action) => {
            let _ = world
                .run_system_cached_with(remove_link_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::MoveNode(action) => {
            let _ = world
                .run_system_cached_with(move_node_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::MoveInput(action) => {
            let _ = world
                .run_system_cached_with(move_input_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::MoveOutput(action) => {
            let _ = world
                .run_system_cached_with(move_output_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::RenameNode(action) => {
            let _ = world
                .run_system_cached_with(rename_node_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::CreateNode(action) => {
            let _ = world
                .run_system_cached_with(create_node_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::EditNode(action) => {
            let _ = world
                .run_system_cached_with(edit_node_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::RemoveNode(action) => {
            let _ = world
                .run_system_cached_with(remove_node_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::UpdateGraphSpec(action) => {
            let _ = world
                .run_system_cached_with(update_node_spec_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::UpdateDefaultData(action) => {
            let _ = world
                .run_system_cached_with(update_input_data_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
        GraphAction::Noop => {}
        GraphAction::GenerateIndices(action) => {
            let _ = world
                .run_system_cached_with(generate_indices_system, action)
                .inspect_err(|err| handle_system_error(err));
        }
    }
}

pub fn create_link_system(In(action): In<CreateLink>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, ctx| {
        if let Ok(()) = graph.can_add_edge(
            Edge {
                source: action.source.clone(),
                target: action.target.clone(),
            },
            ctx,
        ) {
            info!("Adding edge {:?} -> {:?}", action.source, action.target);
            graph.add_edge(action.source, action.target);
        }
    });
    provider.validate(&action.graph);
    provider.generate_indices(&action.graph);
}

pub fn remove_link_system(In(action): In<RemoveLink>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, _| {
        info!("Removing edge with target {:?}", action.target);
        graph.remove_edge_by_target(&action.target);
    });
    provider.validate(&action.graph);
    provider.generate_indices(&action.graph);
}

pub fn move_node_system(In(action): In<MoveNode>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, _| {
        graph
            .editor_metadata
            .set_node_position(action.node, action.new_pos);
    });
    provider.generate_indices(&action.graph);
}

pub fn move_input_system(In(action): In<MoveInput>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, _| {
        graph.editor_metadata.set_input_position(action.new_pos);
    });
    provider.generate_indices(&action.graph);
}

pub fn move_output_system(In(action): In<MoveOutput>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, _| {
        graph.editor_metadata.set_output_position(action.new_pos);
    });
    provider.generate_indices(&action.graph);
}

pub fn rename_node_system(In(action): In<RenameNode>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, _| {
        if let Some(node_mut) = graph.nodes.get_mut(&action.node) {
            info!("Renaming node node {:?}", action.node);
            node_mut.name = action.new_name;
        } else {
            warn!("Cannot rename node {:?}: It does not exist!", action.node);
        }
    });
    provider.validate(&action.graph);
    provider.generate_indices(&action.graph);
}

pub fn create_node_system(In(action): In<CreateNode>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, _| {
        if !graph.nodes.contains_key(&action.node.id) {
            info!("Adding node {:?}", action.node.name);
            graph.add_node(action.node);
        } else {
            warn!("Cannot add node {:?}: Already exists!", action.node.name);
        }
    });
    provider.validate(&action.graph);
    provider.generate_indices(&action.graph);
}

pub fn edit_node_system(In(action): In<EditNode>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, _| {
        if let Some(node_mut) = graph.nodes.get_mut(&action.node) {
            info!("Editing node {:?}", action.node);
            node_mut.inner = action.new_inner;
        } else {
            warn!("Cannot edit node {:?}: It does not exist!", action.node);
        }
    });
    provider.validate(&action.graph);
    provider.generate_indices(&action.graph);
}

pub fn remove_node_system(In(action): In<RemoveNode>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, _| {
        info!("Removing node {:?}", action.node);
        graph.remove_node(action.node);
    });
    provider.validate(&action.graph);
    provider.generate_indices(&action.graph);
}

pub fn update_input_data_system(In(action): In<UpdateDefaultData>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, _| {
        graph.default_data = action.input_data;
    });
    provider.validate(&action.graph);
    provider.generate_indices(&action.graph);
}

pub fn update_node_spec_system(In(action): In<UpdateGraphSpec>, mut provider: GraphAndContext) {
    provider.provide_mut(&action.graph, |graph, _| {
        graph.io_spec = action.new_spec;
    });
    provider.validate(&action.graph);
    provider.generate_indices(&action.graph);
}

pub fn generate_indices_system(In(action): In<GenerateIndices>, mut provider: GraphAndContext) {
    provider.generate_indices(action.graph);
}

fn handle_system_error<Err: Display>(err: Err) {
    error!("Failed to apply graph action: {}", err);
}

#[derive(SystemParam)]
pub struct GraphAndContext<'w> {
    graph_assets: ResMut<'w, Assets<AnimationGraph>>,
    fsm_assets: Res<'w, Assets<StateMachine>>,
    dirty_assets: ResMut<'w, DirtyAssets>,
    graph_indices_map: ResMut<'w, GraphIndicesMap>,
}

impl GraphAndContext<'_> {
    pub fn provide_mut<F>(&mut self, graph_handle: &Handle<AnimationGraph>, f: F)
    where
        F: FnOnce(&mut AnimationGraph, SpecResources),
    {
        self.dirty_assets.add(graph_handle.clone().untyped());

        let graph_assets_copy =
            unsafe { &*(self.graph_assets.as_ref() as *const Assets<AnimationGraph>) };
        let ctx = SpecResources {
            graph_assets: graph_assets_copy,
            fsm_assets: &self.fsm_assets,
        };

        let Some(graph) = self.graph_assets.get_mut(graph_handle) else {
            return;
        };

        f(graph, ctx)
    }

    pub fn provide_ref<F, T>(
        &mut self,
        graph_handle: impl Into<AssetId<AnimationGraph>>,
        f: F,
    ) -> Option<T>
    where
        F: FnOnce(&AnimationGraph, SpecResources) -> Option<T>,
    {
        let graph_assets_copy =
            unsafe { &*(self.graph_assets.as_ref() as *const Assets<AnimationGraph>) };
        let ctx = SpecResources {
            graph_assets: graph_assets_copy,
            fsm_assets: &self.fsm_assets,
        };

        let graph = self.graph_assets.get(graph_handle)?;

        f(graph, ctx)
    }

    pub fn validate(&mut self, graph_handle: &Handle<AnimationGraph>) {
        self.provide_mut(graph_handle, |graph, ctx| {
            while let Err(deletable) = graph.validate_edges(ctx) {
                for Edge { target, .. } in deletable {
                    info!("Removing edge with target {:?}", target);
                    graph.remove_edge_by_target(&target);
                }
            }
        });
    }

    pub fn generate_indices(&mut self, graph_id: impl Into<AssetId<AnimationGraph>>) {
        let graph_id = graph_id.into();
        let indices = self.provide_ref(graph_id, make_graph_indices);
        if let Some(indices) = indices {
            self.graph_indices_map.indices.insert(graph_id, indices);
        }
    }
}
