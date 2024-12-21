use crate::egui_nodes::{
    lib::{PinShape, PinStyleArgs},
    link::{LinkSpec, LinkStyleArgs},
    node::{NodeArgs, NodeSpec},
    pin::{PinSpec, PinType},
};
use bevy::utils::HashMap;
use bevy_animation_graph::core::{
    animation_graph::{AnimationGraph, PinId, SourcePin, TargetPin},
    animation_node::AnimationNode,
    context::{GraphContext, SpecContext},
    edge_data::DataSpec,
};
use bevy_inspector_egui::egui::Color32;

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
    name_to_idx: HashMap<String, usize>,
    idx_to_name: HashMap<usize, String>,
    count: usize,
}

impl Default for NodeIndices {
    fn default() -> Self {
        Self {
            name_to_idx: HashMap::default(),
            idx_to_name: HashMap::default(),
            count: 4, // 0, 1, 2, 3 are reserved for input/output nodes
        }
    }
}

impl NodeIndices {
    pub fn add_mapping(&mut self, name: String) {
        let id = self.count;
        self.count += 1;

        self.name_to_idx.insert(name.clone(), id);
        self.idx_to_name.insert(id, name);
    }

    pub fn id(&self, name: &str) -> Option<usize> {
        self.name_to_idx.get(name).copied()
    }

    pub fn name(&self, id: usize) -> Option<&String> {
        self.idx_to_name.get(&id)
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

pub fn make_graph_indices(
    graph: &AnimationGraph,
    ctx: SpecContext,
) -> Result<GraphIndices, Vec<TargetPin>> {
    let mut graph_indices = GraphIndices::default();

    for node in graph.nodes.values() {
        // add node
        graph_indices.node_indices.add_mapping(node.name.clone());

        // add pins
        for (name, _) in node.inner.data_input_spec(ctx).iter() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Target(TargetPin::NodeData(
                    node.name.clone(),
                    name.clone(),
                )));
        }
        for (name, _) in node.data_output_spec(ctx).iter() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Source(SourcePin::NodeData(
                    node.name.clone(),
                    name.clone(),
                )));
        }
        for (name, _) in node.time_input_spec(ctx).iter() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Target(TargetPin::NodeTime(
                    node.name.clone(),
                    name.clone(),
                )));
        }
        if node.time_output_spec(ctx).is_some() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Source(SourcePin::NodeTime(node.name.clone())));
        }
    }
    // add input/output pins
    for (name, _) in graph.default_parameters.iter() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Source(SourcePin::InputData(name.clone())));
    }
    for (name, _) in graph.input_times.iter() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Source(SourcePin::InputTime(name.clone())));
    }
    for (name, _) in graph.output_parameters.iter() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Target(TargetPin::OutputData(name.clone())));
    }
    if graph.output_time.is_some() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Target(TargetPin::OutputTime));
    }

    let mut remove_edges = vec![];

    // Add edges
    for (target_pin, source_pin) in graph.edges.iter() {
        let Some(source_id) = graph_indices
            .pin_indices
            .id(&Pin::Source(source_pin.clone()))
        else {
            remove_edges.push(target_pin.clone());
            continue;
        };
        let Some(target_id) = graph_indices
            .pin_indices
            .id(&Pin::Target(target_pin.clone()))
        else {
            remove_edges.push(target_pin.clone());
            continue;
        };
        graph_indices.edge_indices.add_mapping(source_id, target_id);
    }

    if remove_edges.is_empty() {
        Ok(graph_indices)
    } else {
        Err(remove_edges)
    }
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
        ctx: SpecContext,
        graph_context: Option<&GraphContext>,
    ) -> Self {
        let mut repr_spec = GraphReprSpec::default();

        repr_spec.add_nodes(graph, indices, ctx, graph_context);
        repr_spec.add_input_output_nodes(graph, indices);
        repr_spec.add_edges(graph, indices, ctx);

        repr_spec
    }

    fn node_debug_info(
        node: &AnimationNode,
        graph_context: Option<&GraphContext>,
    ) -> (Option<f32>, Option<f32>, bool) {
        let Some(graph_context) = graph_context else {
            return (None, None, false);
        };

        let source_pin = SourcePin::NodeTime(node.name.clone());
        let time = graph_context
            .caches
            .get_primary(|c| c.get_time(&source_pin));
        let duration = graph_context
            .caches
            .get_primary(|c| c.get_duration(&source_pin));
        let active = graph_context
            .caches
            .get_primary(|c| Some(c.is_updated(&node.name)));

        (time, duration.flatten(), active.unwrap_or(false))
    }

    fn add_nodes(
        &mut self,
        graph: &AnimationGraph,
        indices: &GraphIndices,
        ctx: SpecContext,
        graph_context: Option<&GraphContext>,
    ) {
        for node in graph.nodes.values() {
            let (time, duration, active) = Self::node_debug_info(node, graph_context);

            let mut constructor = NodeSpec {
                id: indices.node_indices.id(&node.name).unwrap(),
                name: node.name.clone(),
                subtitle: node.display_name(),
                origin: graph
                    .extra
                    .node_positions
                    .get(&node.name)
                    .unwrap()
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

            // parameter input pins
            for (name, spec) in node.data_input_spec(ctx).iter() {
                let (pin_style, _) = param_spec_to_colors(*spec);
                let pin = Pin::Target(TargetPin::NodeData(node.name.clone(), name.clone()));
                let pin_id = indices.pin_indices.id(&pin).unwrap();
                let name = name.clone();
                input_tmp_store.push((
                    name.clone(),
                    PinSpec {
                        id: pin_id,
                        kind: PinType::Input,
                        name,
                        style_args: pin_style,
                        ..Default::default()
                    },
                ));
            }
            // parameter output pins
            for (name, spec) in node.data_output_spec(ctx).iter() {
                let (pin_style, _) = param_spec_to_colors(*spec);
                let pin = Pin::Source(SourcePin::NodeData(node.name.clone(), name.clone()));
                let pin_id = indices.pin_indices.id(&pin).unwrap();
                let name = name.clone();
                output_tmp_store.push((
                    name.clone(),
                    PinSpec {
                        id: pin_id,
                        kind: PinType::Output,
                        name,
                        style_args: pin_style,
                        ..Default::default()
                    },
                ));
            }
            // time input pins
            for (name, _) in node.time_input_spec(ctx).iter() {
                let (pin_style, _) = time_colors();
                let pin = Pin::Target(TargetPin::NodeTime(node.name.clone(), name.clone()));
                let pin_id = indices.pin_indices.id(&pin).unwrap();
                let name = name.clone();
                input_tmp_store.push((
                    name.clone(),
                    PinSpec {
                        id: pin_id,
                        kind: PinType::Input,
                        name,
                        style_args: pin_style,
                        ..Default::default()
                    },
                ));
            }
            // time input pin
            if node.time_output_spec(ctx).is_some() {
                let (pin_style, _) = time_colors();
                let pin = Pin::Source(SourcePin::NodeTime(node.name.clone()));
                let pin_id = indices.pin_indices.id(&pin).unwrap();
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

            let input_order = node.input_pin_ordering(ctx);
            let output_order = node.output_pin_ordering(ctx);

            input_tmp_store.sort_by_key(|(k, _)| input_order.pin_key(k));
            output_tmp_store.sort_by_key(|(k, _)| output_order.pin_key(k));

            constructor
                .attributes
                .extend(input_tmp_store.into_iter().map(|p| p.1));
            constructor
                .attributes
                .extend(output_tmp_store.into_iter().map(|p| p.1));

            self.nodes.push(constructor);
        }
    }

    fn add_input_output_nodes(&mut self, graph: &AnimationGraph, indices: &GraphIndices) {
        // --- Input node
        // ---------------------------------------------------
        let mut input_node_contructor = NodeSpec {
            id: 0,
            args: NodeArgs {
                titlebar: Some(Color32::from_rgb(67, 46, 35)),
                titlebar_hovered: Some(Color32::from_rgb(120, 102, 93)),
                titlebar_selected: Some(Color32::from_rgb(120, 102, 93)),
                ..Default::default()
            },
            name: "Inputs".into(),
            origin: graph.extra.input_position.to_array().into(),
            ..Default::default()
        };

        let mut input_data_store: Vec<(PinId, PinSpec)> = vec![];
        let mut input_time_store: Vec<(PinId, PinSpec)> = vec![];
        let mut output_data_store: Vec<(PinId, PinSpec)> = vec![];

        for (name, p) in graph.default_parameters.iter() {
            let spec = DataSpec::from(p);
            let (pin_style, _) = param_spec_to_colors(spec);
            let pin = Pin::Source(SourcePin::InputData(name.clone()));
            let pin_id = indices.pin_indices.id(&pin).unwrap();
            let name = name.clone();

            input_data_store.push((
                name.clone(),
                PinSpec {
                    id: pin_id,
                    kind: PinType::Output,
                    name,
                    style_args: pin_style,
                    ..Default::default()
                },
            ));
        }

        for (name, _) in graph.input_times.iter() {
            let (pin_style, _) = time_colors();
            let pin = Pin::Source(SourcePin::InputTime(name.clone()));
            let pin_id = indices.pin_indices.id(&pin).unwrap();
            let name = name.clone();
            input_time_store.push((
                name.clone(),
                PinSpec {
                    id: pin_id,
                    kind: PinType::Output,
                    name,
                    style_args: pin_style,
                    ..Default::default()
                },
            ));
        }

        input_data_store
            .sort_by_key(|(k, _)| graph.extra.input_param_order.get(k).copied().unwrap_or(0));
        input_time_store
            .sort_by_key(|(k, _)| graph.extra.input_time_order.get(k).copied().unwrap_or(0));

        input_node_contructor
            .attributes
            .extend(input_data_store.into_iter().map(|p| p.1));
        input_node_contructor
            .attributes
            .extend(input_time_store.into_iter().map(|p| p.1));

        self.nodes.push(input_node_contructor);
        // ---------------------------------------------------

        // --- Output node
        // ---------------------------------------------------
        let mut output_node_contructor = NodeSpec {
            id: 1,
            args: NodeArgs {
                titlebar: Some(Color32::from_rgb(67, 46, 35)),
                titlebar_hovered: Some(Color32::from_rgb(120, 102, 93)),
                titlebar_selected: Some(Color32::from_rgb(120, 102, 93)),
                ..Default::default()
            },
            name: "Outputs".into(),
            origin: graph.extra.output_position.to_array().into(),
            ..Default::default()
        };
        for (name, spec) in graph.output_parameters.iter() {
            let (pin_style, _) = param_spec_to_colors(*spec);
            let pin = Pin::Target(TargetPin::OutputData(name.clone()));
            let pin_id = indices.pin_indices.id(&pin).unwrap();
            let name = name.clone();
            output_data_store.push((
                name.clone(),
                PinSpec {
                    id: pin_id,
                    kind: PinType::Input,
                    name,
                    style_args: pin_style,
                    ..Default::default()
                },
            ));
        }

        if graph.output_time.is_some() {
            let (pin_style, _) = time_colors();
            let pin = Pin::Target(TargetPin::OutputTime);
            let pin_id = indices.pin_indices.id(&pin).unwrap();
            output_node_contructor.attributes.push(PinSpec {
                id: pin_id,
                kind: PinType::Input,
                name: "time".into(),
                style_args: pin_style,
                ..Default::default()
            });
        }

        output_data_store
            .sort_by_key(|(k, _)| graph.extra.output_data_order.get(k).copied().unwrap_or(0));
        output_node_contructor
            .attributes
            .extend(output_data_store.into_iter().map(|p| p.1));

        self.nodes.push(output_node_contructor);
        // ---------------------------------------------------
    }

    fn add_edges(&mut self, graph: &AnimationGraph, indices: &GraphIndices, ctx: SpecContext) {
        for (target_pin, source_pin) in graph.edges.iter() {
            let (edge_id, source_id, target_id) = indices
                .edge_ids(source_pin.clone(), target_pin.clone())
                .unwrap();

            let link_style = match target_pin {
                TargetPin::NodeData(nid, pid) => {
                    let params = graph.nodes.get(nid).unwrap().data_input_spec(ctx);
                    let spec = *params.get(pid).unwrap();
                    param_spec_to_colors(spec).1
                }
                TargetPin::OutputData(pid) => {
                    let spec = graph.output_parameters.get(pid).unwrap();
                    param_spec_to_colors(*spec).1
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
    }
}
