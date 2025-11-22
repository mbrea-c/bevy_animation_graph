use std::any::Any;

use bevy::asset::{AssetId, Assets, Handle};
use bevy_animation_graph::{
    builtin_nodes::EventMarkupNode, core::animation_graph::NodeId, prelude::AnimationGraph,
};
use bevy_inspector_egui::reflect_inspector::InspectorUi;
use egui_dock::egui;

use super::{EguiInspectorExtension, MakeBuffer};
use crate::{impl_widget_hash_from_hash, ui::native_windows::event_track_editor::TargetTracks};

#[derive(Default)]
pub struct TargetTracksInspector;

#[derive(Debug, Default, PartialEq, Eq)]
enum TargetTracksSelectorType {
    #[default]
    Clip,
    GraphNode,
}

impl EguiInspectorExtension for TargetTracksInspector {
    type Base = Option<TargetTracks>;
    type Buffer = TargetTracksBuffer;

    fn mutable(
        value: &mut Self::Base,
        buffer: &mut Self::Buffer,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        mut env: InspectorUi<'_, '_>,
    ) -> bool {
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut buffer.selector_type,
                TargetTracksSelectorType::Clip,
                "Clip",
            );
            ui.radio_value(
                &mut buffer.selector_type,
                TargetTracksSelectorType::GraphNode,
                "Graph node",
            );

            let new_one = match buffer.selector_type {
                TargetTracksSelectorType::Clip => {
                    let mut current =
                        if let Some(TargetTracks::Clip(clip_handle)) = &buffer.selected {
                            clip_handle.clone()
                        } else {
                            Handle::default()
                        };

                    if env.ui_for_reflect_with_options(&mut current, ui, id, options) {
                        Some(TargetTracks::Clip(current))
                    } else {
                        None
                    }
                }
                TargetTracksSelectorType::GraphNode => {
                    let (mut current_handle, mut current_node) =
                        if let Some(TargetTracks::GraphNode { graph, node }) = &mut buffer.selected
                        {
                            (graph.clone(), node.clone())
                        } else {
                            (Handle::default(), NodeId::default())
                        };

                    let changed_handle =
                        env.ui_for_reflect_with_options(&mut current_handle, ui, id, options);

                    let graphs =
                        env.context.world.as_mut().and_then(|world| {
                            world.get_resource_mut::<Assets<AnimationGraph>>().ok()
                        });
                    let nodes_changed = graphs
                        .as_ref()
                        .and_then(|g| {
                            let graph = g.get(current_handle.id());
                            graph.map(|graph| combo_box_nodes(ui, &mut current_node, graph))
                        })
                        .unwrap_or(false);

                    if changed_handle || nodes_changed {
                        buffer.selected = Some(TargetTracks::GraphNode {
                            graph: current_handle.clone(),
                            node: current_node.clone(),
                        });
                        if graphs
                            .map(|g| validate(&g, current_handle.id(), &current_node))
                            .unwrap_or(false)
                        {
                            Some(TargetTracks::GraphNode {
                                graph: current_handle,
                                node: current_node,
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            };

            if let Some(new_one) = new_one {
                buffer.selected = Some(new_one.clone());
                *value = Some(new_one);
                true
            } else {
                false
            }
        })
        .inner
    }

    fn readonly(
        _value: &Self::Base,
        _buffer: &Self::Buffer,
        _ui: &mut egui::Ui,
        _options: &dyn Any,
        _id: egui::Id,
        _env: InspectorUi<'_, '_>,
    ) {
        todo!("not yet supported")
    }
}

fn combo_box_nodes(ui: &mut egui::Ui, current: &mut NodeId, graph: &AnimationGraph) -> bool {
    let mut valid_node_ids = graph
        .nodes
        .values()
        .filter(|n| n.inner.as_any().downcast_ref::<EventMarkupNode>().is_some())
        .map(|n| (n.id, n.name.clone()))
        .collect::<Vec<_>>();

    valid_node_ids.sort_by_key(|(_, n)| n.clone());

    egui::ComboBox::from_id_salt("Select node id for event markup")
        .show_ui(ui, |ui| {
            for (node_id, node_name) in valid_node_ids {
                ui.selectable_value(current, node_id, node_name);
            }
        })
        .response
        .changed()
}

fn validate(
    assets: &Assets<AnimationGraph>,
    graph_id: AssetId<AnimationGraph>,
    node_id: &NodeId,
) -> bool {
    let Some(graph) = assets.get(graph_id) else {
        return false;
    };

    let Some(node) = graph.nodes.get(node_id) else {
        return false;
    };

    node.inner
        .as_any()
        .downcast_ref::<EventMarkupNode>()
        .is_some()
}

impl MakeBuffer<TargetTracksBuffer> for Option<TargetTracks> {
    fn make_buffer(&self) -> TargetTracksBuffer {
        match self {
            Some(TargetTracks::Clip(_)) => TargetTracksBuffer {
                selected: self.clone(),
                selector_type: TargetTracksSelectorType::Clip,
            },
            Some(TargetTracks::GraphNode { .. }) => TargetTracksBuffer {
                selected: self.clone(),
                selector_type: TargetTracksSelectorType::GraphNode,
            },
            None => TargetTracksBuffer {
                selected: None,
                selector_type: TargetTracksSelectorType::Clip,
            },
        }
    }
}

#[derive(Default)]
pub struct TargetTracksBuffer {
    selected: Option<TargetTracks>,
    selector_type: TargetTracksSelectorType,
}

impl_widget_hash_from_hash! { Option<TargetTracks> }
