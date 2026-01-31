use bevy::{asset::AssetId, ecs::resource::Resource, platform::collections::HashMap};
use bevy_animation_graph::core::{
    animation_graph::{AnimationGraph, GraphInputPin, NodeId, PinId, SourcePin, TargetPin},
    animation_node::AnimationNode,
    context::{
        graph_context::GraphState,
        node_states::StateKey,
        spec_context::{IoSpec, NodeInput, NodeOutput, SpecContext, SpecResources},
    },
    edge_data::DataSpec,
    errors::GraphError,
};
use bevy_inspector_egui::egui::Color32;

use crate::egui_nodes::{
    lib::{PinShape, PinStyleArgs},
    link::{LinkSpec, LinkStyleArgs},
    node::{NodeArgs, NodeSpec},
    pin::{PinSpec, PinType},
};

// TODO: Come up with better colors
/// returns (base, hovered, selected)
fn param_spec_to_colors(spec: DataSpec) -> (PinStyleArgs, LinkStyleArgs) {
    match spec {
        DataSpec::F32 => {
            let base = Color32::from_rgb(140, 28, 3);
            let hovered = Color32::from_rgb(184, 99, 74);
            let selected = Color32::from_rgb(184, 99, 74);

            (
                PinStyleArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                    shape: Some(PinShape::CircleFilled),
                },
                LinkStyleArgs {
                    base: Some(base),
                    hovered: Some(hovered),
                    selected: Some(selected),
                    thickness: None,
                },
            )
        }
        DataSpec::Vec2 => {
            let base = Color32::from_rgb(50, 103, 29);
            let hovered = Color32::from_rgb(111, 147, 93);
            let selected = Color32::from_rgb(111, 147, 93);

            (
                PinStyleArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                    shape: Some(PinShape::CircleFilled),
                },
                LinkStyleArgs {
                    base: Some(base),
                    hovered: Some(hovered),
                    selected: Some(selected),
                    thickness: None,
                },
            )
        }
        DataSpec::Vec3 => {
            let base = Color32::from_rgb(50, 103, 29);
            let hovered = Color32::from_rgb(111, 147, 93);
            let selected = Color32::from_rgb(111, 147, 93);

            (
                PinStyleArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                    shape: Some(PinShape::CircleFilled),
                },
                LinkStyleArgs {
                    base: Some(base),
                    hovered: Some(hovered),
                    selected: Some(selected),
                    thickness: None,
                },
            )
        }
        DataSpec::EntityPath => {
            let base = Color32::from_rgb(222, 109, 0);
            let hovered = Color32::from_rgb(241, 153, 89);
            let selected = Color32::from_rgb(241, 153, 89);

            (
                PinStyleArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                    shape: Some(PinShape::CircleFilled),
                },
                LinkStyleArgs {
                    base: Some(base),
                    hovered: Some(hovered),
                    selected: Some(selected),
                    thickness: None,
                },
            )
        }
        DataSpec::Quat => {
            let base = Color32::from_rgb(13, 72, 160);
            let hovered = Color32::from_rgb(108, 122, 189);
            let selected = Color32::from_rgb(108, 122, 189);

            (
                PinStyleArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                    shape: Some(PinShape::CircleFilled),
                },
                LinkStyleArgs {
                    base: Some(base),
                    hovered: Some(hovered),
                    selected: Some(selected),
                    thickness: None,
                },
            )
        }
        DataSpec::BoneMask => {
            let base = Color32::from_rgb(113, 59, 40);
            let hovered = Color32::from_rgb(158, 114, 98);
            let selected = Color32::from_rgb(158, 114, 98);

            (
                PinStyleArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                    shape: Some(PinShape::CircleFilled),
                },
                LinkStyleArgs {
                    base: Some(base),
                    hovered: Some(hovered),
                    selected: Some(selected),
                    thickness: None,
                },
            )
        }
        DataSpec::Pose => {
            let base = Color32::from_rgb(180, 180, 180);
            let hovered = Color32::from_rgb(220, 220, 220);
            let selected = Color32::from_rgb(220, 220, 220);

            (
                PinStyleArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                    shape: Some(PinShape::CircleFilled),
                },
                LinkStyleArgs {
                    base: Some(base),
                    hovered: Some(hovered),
                    selected: Some(selected),
                    thickness: Some(6.),
                },
            )
        }
        DataSpec::EventQueue => {
            let base = Color32::from_rgb(102, 14, 96);
            let hovered = Color32::from_rgb(165, 113, 168);
            let selected = Color32::from_rgb(165, 113, 168);

            (
                PinStyleArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                    shape: Some(PinShape::CircleFilled),
                },
                LinkStyleArgs {
                    base: Some(base),
                    hovered: Some(hovered),
                    selected: Some(selected),
                    thickness: Some(3.),
                },
            )
        }
        DataSpec::Bool => {
            let base = Color32::from_rgb(140, 28, 3);
            let hovered = Color32::from_rgb(184, 99, 74);
            let selected = Color32::from_rgb(184, 99, 74);

            (
                PinStyleArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                    shape: Some(PinShape::CircleFilled),
                },
                LinkStyleArgs {
                    base: Some(base),
                    hovered: Some(hovered),
                    selected: Some(selected),
                    thickness: None,
                },
            )
        }
        DataSpec::RagdollConfig => {
            let base = Color32::from_rgb(28, 140, 3);
            let hovered = Color32::from_rgb(99, 184, 74);
            let selected = Color32::from_rgb(99, 184, 74);
            (
                PinStyleArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                    shape: Some(PinShape::CircleFilled),
                },
                LinkStyleArgs {
                    base: Some(base),
                    hovered: Some(hovered),
                    selected: Some(selected),
                    thickness: Some(6.),
                },
            )
        }
    }
}

/// returns (base, hovered, selected)
fn time_colors() -> (PinStyleArgs, LinkStyleArgs) {
    let base = Color32::from_rgba_unmultiplied(150, 150, 150, 127);
    let hovered = Color32::from_rgb(200, 200, 200);
    let selected = Color32::from_rgb(200, 200, 200);

    (
        PinStyleArgs {
            background: Some(base),
            hovered: Some(hovered),
            shape: Some(PinShape::TriangleFilled),
        },
        LinkStyleArgs {
            base: Some(base),
            hovered: Some(hovered),
            selected: Some(selected),
            thickness: Some(6.),
        },
    )
}

pub struct NodeIndices {
    node_id_to_idx: HashMap<NodeId, usize>,
    idx_to_node_id: HashMap<usize, NodeId>,
    count: usize,
}

impl Default for NodeIndices {
    fn default() -> Self {
        Self {
            node_id_to_idx: HashMap::default(),
            idx_to_node_id: HashMap::default(),
            count: 4, // 0, 1, 2, 3 are reserved for input/output nodes
        }
    }
}

impl NodeIndices {
    pub fn add_mapping(&mut self, name: NodeId) {
        let id = self.count;
        self.count += 1;

        self.node_id_to_idx.insert(name, id);
        self.idx_to_node_id.insert(id, name);
    }

    pub fn id(&self, name: NodeId) -> Option<usize> {
        self.node_id_to_idx.get(&name).copied()
    }

    pub fn name(&self, id: usize) -> Option<NodeId> {
        self.idx_to_node_id.get(&id).copied()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Pin {
    Source(SourcePin),
    Target(TargetPin),
}

#[derive(Default)]
pub struct PinIndices {
    name_to_idx: HashMap<Pin, usize>,
    idx_to_name: HashMap<usize, Pin>,
    count: usize,
}

impl PinIndices {
    pub fn add_mapping(&mut self, pin: Pin) {
        let id = self.count;
        self.count += 1;

        self.name_to_idx.insert(pin.clone(), id);
        self.idx_to_name.insert(id, pin);
    }

    pub fn id(&self, pin: &Pin) -> Option<usize> {
        self.name_to_idx.get(pin).copied()
    }

    pub fn pin(&self, id: usize) -> Option<&Pin> {
        self.idx_to_name.get(&id)
    }
}

#[derive(Default)]
pub struct EdgeIndices {
    name_to_idx: HashMap<(usize, usize), usize>,
    idx_to_name: HashMap<usize, (usize, usize)>,
    count: usize,
}

impl EdgeIndices {
    pub fn add_mapping(&mut self, start_id: usize, end_id: usize) {
        let id = self.count;
        self.count += 1;

        self.name_to_idx.insert((start_id, end_id), id);
        self.idx_to_name.insert(id, (start_id, end_id));
    }

    pub fn id(&self, start_id: usize, end_id: usize) -> Option<usize> {
        self.name_to_idx.get(&(start_id, end_id)).copied()
    }

    pub fn edge(&self, id: usize) -> Option<&(usize, usize)> {
        self.idx_to_name.get(&id)
    }
}

#[derive(Default)]
pub struct GraphIndices {
    pub node_indices: NodeIndices,
    pub pin_indices: PinIndices,
    pub edge_indices: EdgeIndices,
}

impl GraphIndices {
    pub fn edge_ids(
        &self,
        source_pin: SourcePin,
        target_pin: TargetPin,
    ) -> Option<(usize, usize, usize)> {
        let source_id = self.pin_indices.id(&Pin::Source(source_pin))?;
        let target_id = self.pin_indices.id(&Pin::Target(target_pin))?;

        let edge_id = self.edge_indices.id(source_id, target_id)?;

        Some((edge_id, source_id, target_id))
    }
}

#[derive(Resource, Default)]
pub struct GraphIndicesMap {
    pub indices: HashMap<AssetId<AnimationGraph>, GraphIndices>,
}

// TODO: Make an error type that makes sense and use it here
pub fn make_graph_indices(graph: &AnimationGraph, res: SpecResources) -> Option<GraphIndices> {
    let mut graph_indices = GraphIndices::default();

    for node in graph.nodes.values() {
        // add node
        graph_indices.node_indices.add_mapping(node.id);

        let node_spec = node.new_spec(res).ok()?;

        // add pins
        for (name, _) in node_spec.iter_input_data() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Target(TargetPin::NodeData(node.id, name.clone())));
        }
        for (name, _) in node_spec.iter_output_data() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Source(SourcePin::NodeData(node.id, name.clone())));
        }
        for name in node_spec.iter_input_times() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Target(TargetPin::NodeTime(node.id, name.clone())));
        }
        if node_spec.has_output_time() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Source(SourcePin::NodeTime(node.id)));
        }
    }
    // add input/output pins
    for (name, _) in graph.io_spec.iter_input_data() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Source(SourcePin::InputData(name.clone())));
    }
    for name in graph.io_spec.iter_input_times() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Source(SourcePin::InputTime(name.clone())));
    }
    for (name, _) in graph.io_spec.iter_output_data() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Target(TargetPin::OutputData(name.clone())));
    }
    if graph.io_spec.has_output_time() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Target(TargetPin::OutputTime));
    }

    // Add edges
    for (target_pin, source_pin) in graph.edges_inverted.iter() {
        let source_id = graph_indices
            .pin_indices
            .id(&Pin::Source(source_pin.clone()))?;
        let target_id = graph_indices
            .pin_indices
            .id(&Pin::Target(target_pin.clone()))?;

        graph_indices.edge_indices.add_mapping(source_id, target_id);
    }

    Some(graph_indices)
}

#[derive(Default)]
pub struct GraphReprSpec {
    pub nodes: Vec<NodeSpec>,
    pub edges: Vec<LinkSpec>,
}

impl GraphReprSpec {
    pub fn from_graph(
        graph: &AnimationGraph,
        indices: &GraphIndices,
        ctx: SpecResources,
        graph_context: Option<&GraphState>,
    ) -> Option<Self> {
        let mut repr_spec = GraphReprSpec::default();

        repr_spec.add_nodes(graph, indices, ctx, graph_context)?;
        repr_spec.add_input_output_nodes(graph, indices)?;
        let _ = repr_spec.add_edges(graph, indices, ctx);

        Some(repr_spec)
    }

    fn node_debug_info(
        node: &AnimationNode,
        graph_context: Option<&GraphState>,
    ) -> (Option<f32>, Option<f32>, bool) {
        let Some(graph_context) = graph_context else {
            return (None, None, false);
        };

        let time = graph_context
            .node_states
            .get_time(node.id, StateKey::Default);
        let duration = graph_context
            .node_caches
            .get_duration(node.id, StateKey::Default)
            .ok();
        let active = graph_context
            .node_caches
            .is_updated(node.id, StateKey::Default);

        (Some(time), duration.flatten(), active)
    }

    fn add_nodes(
        &mut self,
        graph: &AnimationGraph,
        indices: &GraphIndices,
        resources: SpecResources,
        graph_context: Option<&GraphState>,
    ) -> Option<()> {
        for node in graph.nodes.values() {
            let (time, duration, active) = Self::node_debug_info(node, graph_context);

            let mut constructor = NodeSpec {
                id: indices.node_indices.id(node.id)?,
                name: node.name.clone(),
                subtitle: node.display_name(),
                origin: graph
                    .editor_metadata
                    .node_positions
                    .get(&node.id)
                    .copied()
                    .unwrap_or_default()
                    .to_array()
                    .into(),
                time,
                duration,
                active,
                ..Default::default()
            };

            // We first insert into a temporary store so that we can sort pins
            let mut input_tmp_store: Vec<(PinId, PinSpec)> = vec![];
            let mut output_tmp_store: Vec<(PinId, PinSpec)> = vec![];

            let mut node_spec = IoSpec::<String>::default();
            let ctx = SpecContext::new(resources, &mut node_spec);
            let _ = node.spec(ctx);

            for input in node_spec.sorted_inputs().into_iter() {
                match input {
                    NodeInput::Time(pin_id) => {
                        let (pin_style, _) = time_colors();
                        let pin = Pin::Target(TargetPin::NodeTime(node.id, pin_id.clone()));
                        let pin_idx = indices.pin_indices.id(&pin)?;
                        let name = pin_id.clone();
                        input_tmp_store.push((
                            name.clone(),
                            PinSpec {
                                id: pin_idx,
                                kind: PinType::Input,
                                name,
                                style_args: pin_style,
                                ..Default::default()
                            },
                        ));
                    }
                    NodeInput::Data(pin_id, data_spec) => {
                        let (pin_style, _) = param_spec_to_colors(data_spec);
                        let pin = Pin::Target(TargetPin::NodeData(node.id, pin_id.clone()));
                        let pin_idx = indices.pin_indices.id(&pin)?;
                        let name = pin_id.clone();
                        input_tmp_store.push((
                            name.clone(),
                            PinSpec {
                                id: pin_idx,
                                kind: PinType::Input,
                                name,
                                style_args: pin_style,
                                ..Default::default()
                            },
                        ));
                    }
                }
            }

            for output in node_spec.sorted_outputs().into_iter() {
                match output {
                    NodeOutput::Time => {
                        let (pin_style, _) = time_colors();
                        let pin = Pin::Source(SourcePin::NodeTime(node.id));
                        let pin_id = indices.pin_indices.id(&pin)?;
                        output_tmp_store.push((
                            "___time_out".into(),
                            PinSpec {
                                id: pin_id,
                                kind: PinType::Output,
                                name: "time".into(),
                                style_args: pin_style,
                                ..Default::default()
                            },
                        ));
                    }
                    NodeOutput::Data(pin_id, data_spec) => {
                        let (pin_style, _) = param_spec_to_colors(data_spec);
                        let pin = Pin::Source(SourcePin::NodeData(node.id, pin_id.clone()));
                        let pin_idx = indices.pin_indices.id(&pin)?;
                        let name = pin_id.clone();
                        output_tmp_store.push((
                            name.clone(),
                            PinSpec {
                                id: pin_idx,
                                kind: PinType::Output,
                                name,
                                style_args: pin_style,
                                ..Default::default()
                            },
                        ));
                    }
                }
            }

            constructor
                .attributes
                .extend(input_tmp_store.into_iter().map(|p| p.1));
            constructor
                .attributes
                .extend(output_tmp_store.into_iter().map(|p| p.1));

            self.nodes.push(constructor);
        }

        Some(())
    }

    fn add_input_output_nodes(
        &mut self,
        graph: &AnimationGraph,
        indices: &GraphIndices,
    ) -> Option<()> {
        // --- Input node
        let mut input_node_contructor = NodeSpec {
            id: 0,
            args: NodeArgs {
                titlebar: Some(Color32::from_rgb(67, 46, 35)),
                titlebar_hovered: Some(Color32::from_rgb(120, 102, 93)),
                titlebar_selected: Some(Color32::from_rgb(120, 102, 93)),
                ..Default::default()
            },
            name: "Inputs".into(),
            origin: graph.editor_metadata.input_position.to_array().into(),
            ..Default::default()
        };

        for input in graph.io_spec.sorted_inputs() {
            match input {
                NodeInput::Time(pin_id) => {
                    let (pin_style, _) = time_colors();
                    let pin = Pin::Source(SourcePin::InputTime(pin_id.clone()));
                    let pin_idx = indices.pin_indices.id(&pin)?;
                    input_node_contructor.attributes.push(PinSpec {
                        id: pin_idx,
                        kind: PinType::Output,
                        name: graph_input_pin_string(&pin_id),
                        style_args: pin_style,
                        ..Default::default()
                    });
                }
                NodeInput::Data(pin_id, data_spec) => {
                    let (pin_style, _) = param_spec_to_colors(data_spec);
                    let pin = Pin::Source(SourcePin::InputData(pin_id.clone()));
                    let pin_idx = indices.pin_indices.id(&pin)?;
                    input_node_contructor.attributes.push(PinSpec {
                        id: pin_idx,
                        kind: PinType::Output,
                        name: graph_input_pin_string(&pin_id),
                        style_args: pin_style,
                        ..Default::default()
                    });
                }
            }
        }

        self.nodes.push(input_node_contructor);

        // --- Output node
        let mut output_node_contructor = NodeSpec {
            id: 1,
            args: NodeArgs {
                titlebar: Some(Color32::from_rgb(67, 46, 35)),
                titlebar_hovered: Some(Color32::from_rgb(120, 102, 93)),
                titlebar_selected: Some(Color32::from_rgb(120, 102, 93)),
                ..Default::default()
            },
            name: "Outputs".into(),
            origin: graph.editor_metadata.output_position.to_array().into(),
            ..Default::default()
        };

        for output in graph.io_spec.sorted_outputs() {
            match output {
                NodeOutput::Time => {
                    let (pin_style, _) = time_colors();
                    let pin = Pin::Target(TargetPin::OutputTime);
                    let pin_id = indices.pin_indices.id(&pin)?;
                    output_node_contructor.attributes.push(PinSpec {
                        id: pin_id,
                        kind: PinType::Input,
                        name: "time".into(),
                        style_args: pin_style,
                        ..Default::default()
                    });
                }
                NodeOutput::Data(pin_id, data_spec) => {
                    let (pin_style, _) = param_spec_to_colors(data_spec);
                    let pin = Pin::Target(TargetPin::OutputData(pin_id.clone()));
                    let pin_idx = indices.pin_indices.id(&pin)?;
                    output_node_contructor.attributes.push(PinSpec {
                        id: pin_idx,
                        kind: PinType::Input,
                        name: pin_id,
                        style_args: pin_style,
                        ..Default::default()
                    });
                }
            }
        }

        self.nodes.push(output_node_contructor);
        Some(())
    }

    fn add_edges(
        &mut self,
        graph: &AnimationGraph,
        indices: &GraphIndices,
        ctx: SpecResources,
    ) -> Result<(), GraphError> {
        for (target_pin, source_pin) in graph.edges_inverted.iter() {
            let (edge_id, source_id, target_id) = indices
                .edge_ids(source_pin.clone(), target_pin.clone())
                .unwrap();

            let link_style = match target_pin {
                TargetPin::NodeData(nid, pid) => {
                    let params = graph.nodes.get(nid).unwrap().new_spec(ctx)?;
                    let spec = params.get_input_data(pid).unwrap();
                    param_spec_to_colors(spec).1
                }
                TargetPin::OutputData(pid) => {
                    let spec = graph.io_spec.get_output_data(pid).unwrap();
                    param_spec_to_colors(spec).1
                }
                TargetPin::NodeTime(_, _) => time_colors().1,
                TargetPin::OutputTime => time_colors().1,
            };

            self.edges.push(LinkSpec {
                id: edge_id,
                start_pin_index: source_id,
                end_pin_index: target_id,
                style: link_style,
            });
        }

        Ok(())
    }
}

fn graph_input_pin_string(input: &GraphInputPin) -> String {
    match input {
        GraphInputPin::Passthrough(pin_id) => pin_id.clone(),
        GraphInputPin::FromFsmSource(pin_id) => format!("src: {}", pin_id),
        GraphInputPin::FromFsmTarget(pin_id) => format!("tgt: {}", pin_id),
        GraphInputPin::FsmBuiltin(fsm_builtin_pin) => format!("fsm: {:?}", fsm_builtin_pin),
    }
}
