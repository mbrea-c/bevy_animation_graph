use crate::egui_nodes::{
    lib::{PinArgs, PinShape},
    link::{LinkColorArgs, LinkSpec},
    node::{NodeArgs, NodeSpec},
    pin::{PinSpec, PinType},
};
use bevy::utils::HashMap;
use bevy_animation_graph::core::{
    animation_graph::{AnimationGraph, SourcePin, TargetPin},
    animation_node::NodeLike,
    context::SpecContext,
    frame::PoseSpec,
    parameters::ParamSpec,
};
use bevy_inspector_egui::egui::Color32;

/// returns (base, hovered, selected)
fn param_spec_to_colors(spec: ParamSpec) -> (Color32, Color32, Color32) {
    match spec {
        ParamSpec::F32 => (
            Color32::from_rgb(140, 28, 3),
            Color32::from_rgb(184, 99, 74),
            Color32::from_rgb(184, 99, 74),
        ),
        ParamSpec::Vec3 => (
            Color32::from_rgb(50, 103, 29),
            Color32::from_rgb(111, 147, 93),
            Color32::from_rgb(111, 147, 93),
        ),
        ParamSpec::EntityPath => (
            Color32::from_rgb(222, 109, 0),
            Color32::from_rgb(241, 153, 89),
            Color32::from_rgb(241, 153, 89),
        ),
        ParamSpec::Quat => (
            Color32::from_rgb(13, 72, 160),
            Color32::from_rgb(108, 122, 189),
            Color32::from_rgb(108, 122, 189),
        ),
        ParamSpec::BoneMask => (
            Color32::from_rgb(113, 59, 40),
            Color32::from_rgb(158, 114, 98),
            Color32::from_rgb(158, 114, 98),
        ),
    }
}

/// returns (base, hovered, selected)
fn pose_spec_to_colors(_spec: PoseSpec) -> (Color32, Color32, Color32) {
    (
        Color32::from_rgb(180, 180, 180),
        Color32::from_rgb(220, 220, 220),
        Color32::from_rgb(220, 220, 220),
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
        for (name, _) in node.parameter_input_spec(ctx).iter() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Target(TargetPin::NodeParameter(
                    node.name.clone(),
                    name.clone(),
                )));
        }
        for (name, _) in node.parameter_output_spec(ctx).iter() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Source(SourcePin::NodeParameter(
                    node.name.clone(),
                    name.clone(),
                )));
        }
        for (name, _) in node.pose_input_spec(ctx).iter() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Target(TargetPin::NodePose(
                    node.name.clone(),
                    name.clone(),
                )));
        }
        if node.pose_output_spec(ctx).is_some() {
            graph_indices
                .pin_indices
                .add_mapping(Pin::Source(SourcePin::NodePose(node.name.clone())));
        }
    }
    // add input/output pins
    for (name, _) in graph.default_parameters.iter() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Source(SourcePin::InputParameter(name.clone())));
    }
    for (name, _) in graph.input_poses.iter() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Source(SourcePin::InputPose(name.clone())));
    }
    for (name, _) in graph.output_parameters.iter() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Target(TargetPin::OutputParameter(name.clone())));
    }
    if graph.output_pose.is_some() {
        graph_indices
            .pin_indices
            .add_mapping(Pin::Target(TargetPin::OutputPose));
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
    pub fn from_graph(graph: &AnimationGraph, indices: &GraphIndices, ctx: SpecContext) -> Self {
        let mut repr_spec = GraphReprSpec::default();

        repr_spec.add_nodes(graph, indices, ctx);
        repr_spec.add_input_output_nodes(graph, indices);
        repr_spec.add_edges(graph, indices, ctx);

        repr_spec
    }

    fn add_nodes(&mut self, graph: &AnimationGraph, indices: &GraphIndices, ctx: SpecContext) {
        for node in graph.nodes.values() {
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
                ..Default::default()
            };

            // parameter input pins
            for (name, spec) in node.parameter_input_spec(ctx).iter() {
                let spec = spec.spec;
                let (base, hovered, _) = param_spec_to_colors(spec);
                let pin = Pin::Target(TargetPin::NodeParameter(node.name.clone(), name.clone()));
                let pin_id = indices.pin_indices.id(&pin).unwrap();
                let name = name.clone();
                constructor.attributes.push(PinSpec {
                    id: pin_id,
                    kind: PinType::Input,
                    name,
                    style_args: PinArgs {
                        background: Some(base),
                        hovered: Some(hovered),
                    },
                    ..Default::default()
                });
            }
            // parameter output pins
            for (name, spec) in node.parameter_output_spec(ctx).iter() {
                let (base, hovered, _) = param_spec_to_colors(*spec);
                let pin = Pin::Source(SourcePin::NodeParameter(node.name.clone(), name.clone()));
                let pin_id = indices.pin_indices.id(&pin).unwrap();
                let name = name.clone();
                constructor.attributes.push(PinSpec {
                    id: pin_id,
                    kind: PinType::Output,
                    name,
                    style_args: PinArgs {
                        background: Some(base),
                        hovered: Some(hovered),
                    },
                    ..Default::default()
                });
            }
            // pose input pins
            for (name, spec) in node.pose_input_spec(ctx).iter() {
                let (base, hovered, _) = pose_spec_to_colors(*spec);
                let pin = Pin::Target(TargetPin::NodePose(node.name.clone(), name.clone()));
                let pin_id = indices.pin_indices.id(&pin).unwrap();
                let name = name.clone();
                constructor.attributes.push(PinSpec {
                    id: pin_id,
                    kind: PinType::Input,
                    shape: PinShape::TriangleFilled,
                    name,
                    style_args: PinArgs {
                        background: Some(base),
                        hovered: Some(hovered),
                    },
                    ..Default::default()
                });
            }
            // pose input pin
            if let Some(spec) = node.pose_output_spec(ctx) {
                let (base, hovered, _) = pose_spec_to_colors(spec);
                let pin = Pin::Source(SourcePin::NodePose(node.name.clone()));
                let pin_id = indices.pin_indices.id(&pin).unwrap();
                constructor.attributes.push(PinSpec {
                    id: pin_id,
                    kind: PinType::Output,
                    shape: PinShape::TriangleFilled,
                    name: "Pose".into(),
                    style_args: PinArgs {
                        background: Some(base),
                        hovered: Some(hovered),
                    },
                    ..Default::default()
                });
            }

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
        for (name, p) in graph.default_parameters.iter() {
            let spec = ParamSpec::from(p);
            let (base, hovered, _) = param_spec_to_colors(spec);
            let pin = Pin::Source(SourcePin::InputParameter(name.clone()));
            let pin_id = indices.pin_indices.id(&pin).unwrap();
            let name = name.clone();

            input_node_contructor.attributes.push(PinSpec {
                id: pin_id,
                kind: PinType::Output,
                name,
                style_args: PinArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                },
                ..Default::default()
            });
        }

        for (name, spec) in graph.input_poses.iter() {
            let (base, hovered, _) = pose_spec_to_colors(*spec);
            let pin = Pin::Source(SourcePin::InputPose(name.clone()));
            let pin_id = indices.pin_indices.id(&pin).unwrap();
            let name = name.clone();
            input_node_contructor.attributes.push(PinSpec {
                id: pin_id,
                kind: PinType::Output,
                shape: PinShape::TriangleFilled,
                name,
                style_args: PinArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                },
                ..Default::default()
            });
        }

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
            let (base, hovered, _) = param_spec_to_colors(*spec);
            let pin = Pin::Target(TargetPin::OutputParameter(name.clone()));
            let pin_id = indices.pin_indices.id(&pin).unwrap();
            let name = name.clone();
            output_node_contructor.attributes.push(PinSpec {
                id: pin_id,
                kind: PinType::Input,
                name,
                style_args: PinArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                },
                ..Default::default()
            });
        }

        if let Some(spec) = graph.output_pose {
            let (base, hovered, _) = pose_spec_to_colors(spec);
            let pin = Pin::Target(TargetPin::OutputPose);
            let pin_id = indices.pin_indices.id(&pin).unwrap();
            output_node_contructor.attributes.push(PinSpec {
                id: pin_id,
                kind: PinType::Input,
                shape: PinShape::TriangleFilled,
                name: "Pose".into(),
                style_args: PinArgs {
                    background: Some(base),
                    hovered: Some(hovered),
                },
                ..Default::default()
            });
        }

        self.nodes.push(output_node_contructor);
        // ---------------------------------------------------
    }

    fn add_edges(&mut self, graph: &AnimationGraph, indices: &GraphIndices, ctx: SpecContext) {
        for (target_pin, source_pin) in graph.edges.iter() {
            let (edge_id, source_id, target_id) = indices
                .edge_ids(source_pin.clone(), target_pin.clone())
                .unwrap();

            let (color, thickness) = match target_pin {
                TargetPin::NodeParameter(nid, pid) => {
                    let params = graph.nodes.get(nid).unwrap().parameter_input_spec(ctx);
                    let spec = params.get(pid).unwrap().spec;
                    let (base, hovered, selected) = param_spec_to_colors(spec);
                    (
                        LinkColorArgs {
                            base: Some(base),
                            hovered: Some(hovered),
                            selected: Some(selected),
                        },
                        3.,
                    )
                }
                TargetPin::OutputParameter(pid) => {
                    let spec = graph.output_parameters.get(pid).unwrap();
                    let (base, hovered, selected) = param_spec_to_colors(*spec);
                    (
                        LinkColorArgs {
                            base: Some(base),
                            hovered: Some(hovered),
                            selected: Some(selected),
                        },
                        3.,
                    )
                }
                TargetPin::NodePose(nid, pid) => {
                    let poses = graph.nodes.get(nid).unwrap().pose_input_spec(ctx);
                    let spec = poses.get(pid).unwrap();
                    let (base, hovered, selected) = pose_spec_to_colors(*spec);
                    (
                        LinkColorArgs {
                            base: Some(base),
                            hovered: Some(hovered),
                            selected: Some(selected),
                        },
                        5.,
                    )
                }
                TargetPin::OutputPose => {
                    let spec = graph.output_pose.as_ref().unwrap();
                    let (base, hovered, selected) = pose_spec_to_colors(*spec);
                    (
                        LinkColorArgs {
                            base: Some(base),
                            hovered: Some(hovered),
                            selected: Some(selected),
                        },
                        5.,
                    )
                }
            };

            self.edges.push(LinkSpec {
                id: edge_id,
                start_pin_index: source_id,
                end_pin_index: target_id,
                thickness: Some(thickness),
                color_style: color,
            });
        }
    }
}
