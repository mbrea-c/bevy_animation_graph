use super::{AnimationGraph, SourcePin, TimeState, TimeUpdate};
use crate::{
    core::{
        animation_node::NodeLike,
        frame::{BoneFrame, PoseFrame, ValueFrame},
        graph_context::{GraphContext, GraphContextTmp},
    },
    nodes::{ClipNode, GraphNode},
    prelude::{OptParamSpec, ParamSpec, ParamValue, SpecContext},
};
use bevy::{
    reflect::{FromReflect, TypePath},
    utils::HashMap,
};
use std::{
    fs::File,
    io::BufWriter,
    process::{Command, Stdio},
};

pub trait ToDot {
    fn to_dot(
        &self,
        f: &mut impl std::io::Write,
        context: Option<&mut GraphContext>,
        context_tmp: GraphContextTmp,
    ) -> std::io::Result<()>;

    fn preview_dot(
        &self,
        context: Option<&mut GraphContext>,
        context_tmp: GraphContextTmp,
    ) -> std::io::Result<()> {
        let dir = std::env::temp_dir();
        let path = dir.join("bevy_animation_graph_dot.dot");

        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        self.to_dot(&mut writer, context, context_tmp)?;
        writer.get_mut().sync_all()?;

        let dot = Command::new("dot")
            .args([path.to_str().unwrap(), "-Tpdf", "-O"])
            .stdout(Stdio::piped())
            .spawn()?;
        Command::new("zathura")
            .args(["-"])
            .stdin(Stdio::from(dot.stdout.unwrap()))
            .spawn()?;

        Ok(())
    }

    fn dot_to_tmp_file_and_open(
        &self,
        context: Option<&mut GraphContext>,
        context_tmp: GraphContextTmp,
    ) -> std::io::Result<()> {
        self.dot_to_tmp_file(context, context_tmp)?;

        Command::new("zathura")
            .args(["/tmp/bevy_animation_graph_dot.dot.pdf"])
            .spawn()?;

        Ok(())
    }

    fn dot_to_file(
        &self,
        path: &str,
        context: Option<&mut GraphContext>,
        context_tmp: GraphContextTmp,
    ) -> std::io::Result<()> {
        {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            self.to_dot(&mut writer, context, context_tmp)?;
        }

        Ok(())
    }

    fn dot_to_stdout(
        &self,
        context: Option<&mut GraphContext>,
        context_tmp: GraphContextTmp,
    ) -> std::io::Result<()> {
        {
            let mut stdout = std::io::stdout();
            self.to_dot(&mut stdout, context, context_tmp)?;
        }

        Ok(())
    }

    fn dot_to_tmp_file(
        &self,
        context: Option<&mut GraphContext>,
        context_tmp: GraphContextTmp,
    ) -> std::io::Result<()> {
        let path = "/tmp/bevy_animation_graph_dot.dot";
        let pdf_path = "/tmp/bevy_animation_graph_dot.dot.pdf";
        let pdf_path_alt = "/tmp/bevy_animation_graph_dot.dot.pdf_alt";

        {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            self.to_dot(&mut writer, context, context_tmp)?;
        }

        {
            let pdf_file_alt = File::create(pdf_path_alt)?;
            Command::new("dot")
                .args([path, "-Tpdf"])
                .stdout(pdf_file_alt)
                .status()?;

            std::fs::rename(pdf_path_alt, pdf_path)?;
        }

        Ok(())
    }
}

fn write_col(
    f: &mut impl std::io::Write,
    row: HashMap<String, OptParamSpec>,
) -> std::io::Result<()> {
    if !row.is_empty() {
        write!(f, "<TABLE BORDER=\"0\">")?;
        for (param_name, param_spec) in row.iter() {
            let icon = match param_spec.spec {
                ParamSpec::F32 => String::from(""),
                ParamSpec::BoneMask => String::from("󰚌"),
                ParamSpec::Quat => String::from("󰑵"),
            };

            write!(
                f,
                "<TR><TD PORT=\"{}\">{} {}</TD></TR>",
                param_name, icon, param_name
            )?;
        }
        write!(f, "</TABLE>")?;
    }
    Ok(())
}

fn write_col_pose(f: &mut impl std::io::Write, row: HashMap<String, ()>) -> std::io::Result<()> {
    if !row.is_empty() {
        write!(f, "<TABLE BORDER=\"0\">")?;
        for (param_name, _) in row.iter() {
            let icon = String::from("🯅");
            write!(
                f,
                "<TR><TD PORT=\"{}\">{} {}</TD></TR>",
                param_name, icon, param_name
            )?;
        }
        write!(f, "</TABLE>")?;
    }
    Ok(())
}

fn write_rows(
    f: &mut impl std::io::Write,
    left: HashMap<String, OptParamSpec>,
    right: HashMap<String, OptParamSpec>,
) -> std::io::Result<()> {
    write!(f, "<TR>")?;
    write!(f, "<TD>")?;
    write_col(f, left)?;
    write!(f, "</TD>")?;
    write!(f, "<TD>")?;
    write_col(f, right)?;
    write!(f, "</TD>")?;
    write!(f, "</TR>")?;
    Ok(())
}

fn write_rows_pose(
    f: &mut impl std::io::Write,
    left: HashMap<String, ()>,
    right: HashMap<String, ()>,
) -> std::io::Result<()> {
    write!(f, "<TR>")?;
    write!(f, "<TD>")?;
    write_col_pose(f, left)?;
    write!(f, "</TD>")?;
    write!(f, "<TD>")?;
    write_col_pose(f, right)?;
    write!(f, "</TD>")?;
    write!(f, "</TR>")?;
    Ok(())
}

fn write_debug_info(f: &mut impl std::io::Write, pose: PoseFrame) -> std::io::Result<()> {
    write!(f, "<TR>")?;
    write!(f, "<TD COLSPAN=\"2\">")?;
    if pose.verify_timestamps_in_order() {
        write!(f, " Timestamps not in order<BR/>")?;
    }
    if pose.verify_timestamp_in_range() {
        write!(f, " Timestamp not in range<BR/>")?;
    }
    write!(f, "</TD>")?;
    write!(f, "</TR>")?;
    Ok(())
}

pub trait AsDotLabel {
    fn as_dot_label(&self) -> String;
}

impl AsDotLabel for ParamValue {
    fn as_dot_label(&self) -> String {
        match self {
            ParamValue::F32(f) => format!("{:.3}", f),
            ParamValue::Quat(q) => format!("{}", q),
            ParamValue::BoneMask(_) => format!("Bone Mask"),
        }
    }
}

impl AsDotLabel for PoseFrame {
    fn as_dot_label(&self) -> String {
        self.bones
            .iter()
            .map(|b| b.as_dot_label())
            .collect::<Vec<String>>()
            .join("<br/>")
    }
}

impl AsDotLabel for BoneFrame {
    fn as_dot_label(&self) -> String {
        self.rotation
            .as_ref()
            .map_or("".into(), |r| r.as_dot_label())
    }
}

impl<T: FromReflect + TypePath> AsDotLabel for ValueFrame<T> {
    fn as_dot_label(&self) -> String {
        format!("{:.3}<->{:.3}", self.prev_timestamp, self.next_timestamp)
    }
}

impl AsDotLabel for Option<f32> {
    fn as_dot_label(&self) -> String {
        format!("{:?}", self)
    }
}

impl AsDotLabel for f32 {
    fn as_dot_label(&self) -> String {
        format!("{:.3}", self)
    }
}

impl AsDotLabel for TimeUpdate {
    fn as_dot_label(&self) -> String {
        match self {
            TimeUpdate::Delta(dt) => format!("Δt({:.3})", dt),
            TimeUpdate::Absolute(t) => format!("t🡠{:.3}", t),
        }
    }
}

impl AsDotLabel for TimeState {
    fn as_dot_label(&self) -> String {
        format!("{:.3} after {}", self.time, self.update.as_dot_label())
    }
}

impl ToDot for AnimationGraph {
    fn to_dot(
        &self,
        f: &mut impl std::io::Write,
        mut context: Option<&mut GraphContext>,
        context_tmp: GraphContextTmp,
    ) -> std::io::Result<()> {
        writeln!(f, "digraph {{")?;
        writeln!(f, "\trankdir=LR;")?;
        writeln!(f, "\tnode [style=rounded, shape=plain];")?;

        let mut default_graph_context = GraphContext::default();

        let ctx = if let Some(context) = &mut context {
            context
        } else {
            &mut default_graph_context
        };

        for (name, node) in self.nodes.iter() {
            write!(
                f,
                "\t\"{}\" [label=<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">",
                name
            )?;
            write!(
                f,
                "<TR><TD COLSPAN=\"2\"><B>{}</B><BR/><i>{}</i>",
                name,
                node.display_name()
            )?;

            match &node.node {
                crate::core::animation_node::AnimationNodeType::Clip(ClipNode { clip, .. }) => {
                    write!(
                        f,
                        "<br/><sub><i>{}</i></sub><br/><br/>",
                        clip.path().unwrap()
                    )?;
                }
                crate::core::animation_node::AnimationNodeType::Graph(GraphNode {
                    graph, ..
                }) => {
                    write!(
                        f,
                        "<br/><sub><i>{}</i></sub><br/><br/>",
                        graph.path().unwrap()
                    )?;
                }
                _ => {}
            };
            write!(f, "</TD></TR>",)?;

            let in_param = node.parameter_input_spec(SpecContext::new(ctx, context_tmp));
            let out_param = node.parameter_output_spec(SpecContext::new(ctx, context_tmp));

            let in_td = node.pose_input_spec(SpecContext::new(ctx, context_tmp));
            let out_td = node.pose_output_spec(SpecContext::new(ctx, context_tmp));

            write_rows(
                f,
                in_param.into_iter().map(|(k, v)| (k, v)).collect(),
                out_param.into_iter().map(|(k, v)| (k, v.into())).collect(),
            )?;

            let mut right = HashMap::new();
            if out_td {
                right.insert("POSE".into(), ());
            }

            write_rows_pose(f, in_td.into_iter().map(|k| (k, ())).collect(), right)?;

            if let Some(frame) = ctx.get_pose(&SourcePin::NodePose(name.clone())) {
                write_debug_info(f, frame.clone())?;
            }

            writeln!(f, "</TABLE>>]")?;
        }

        // --- Input parameters node
        // --------------------------------------------------------
        let name = "INPUT PARAMETERS";
        write!(
            f,
            "\t\"{}\" [label=<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">",
            name
        )?;
        write!(f, "<TR><TD COLSPAN=\"2\"><B>{}</B>", name)?;
        write!(f, "</TD></TR>",)?;
        let out_param = self.input_parameters.clone();
        write_rows(f, HashMap::new(), out_param)?;
        writeln!(f, "</TABLE>>]")?;
        // --------------------------------------------------------

        // --- Input poses node
        // --------------------------------------------------------
        let name = "INPUT POSES";
        write!(
            f,
            "\t\"{}\" [label=<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">",
            name
        )?;
        write!(f, "<TR><TD COLSPAN=\"2\"><B>{}</B>", name)?;
        write!(f, "</TD></TR>",)?;
        let out_param = self.input_poses.clone();
        write_rows_pose(
            f,
            HashMap::new(),
            out_param.into_iter().map(|k| (k, ())).collect(),
        )?;
        writeln!(f, "</TABLE>>]")?;
        // --------------------------------------------------------

        // --- Output parameters node
        // --------------------------------------------------------
        let name = "OUTPUT PARAMETERS";
        write!(
            f,
            "\t\"{}\" [label=<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">",
            name
        )?;
        write!(f, "<TR><TD COLSPAN=\"2\"><B>{}</B>", name)?;
        write!(f, "</TD></TR>",)?;
        let out_param = self.output_parameters.clone();
        write_rows(
            f,
            out_param.into_iter().map(|(k, v)| (k, v.into())).collect(),
            HashMap::new(),
        )?;
        writeln!(f, "</TABLE>>]")?;
        // --------------------------------------------------------

        // --- Output pose node
        // --------------------------------------------------------
        let name = "OUTPUT POSE";
        write!(
            f,
            "\t\"{}\" [label=<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">",
            name
        )?;
        write!(f, "<TR><TD COLSPAN=\"2\"><B>{}</B>", name)?;
        write!(f, "</TD></TR>",)?;
        let out_param = self.output_pose;

        let mut out = HashMap::new();
        if out_param {
            out.insert("POSE".into(), ());
        }
        write_rows_pose(f, out, HashMap::new())?;
        writeln!(f, "</TABLE>>]")?;
        // --------------------------------------------------------

        for (target_pin, source_pin) in self.edges.iter() {
            let (start_node, start_edge) = match source_pin {
                super::SourcePin::NodeParameter(node_id, pin_id) => {
                    (node_id.clone(), pin_id.clone())
                }
                super::SourcePin::InputParameter(pin_id) => {
                    (String::from("INPUT PARAMETERS"), pin_id.clone())
                }
                super::SourcePin::NodePose(node_id) => (node_id.clone(), String::from("POSE")),
                super::SourcePin::InputPose(pin_id) => {
                    (String::from("INPUT POSES"), pin_id.clone())
                }
            };

            let (end_node, end_edge) = match target_pin {
                super::TargetPin::NodeParameter(node_id, pin_id) => {
                    (node_id.clone(), pin_id.clone())
                }
                super::TargetPin::OutputParameter(pin_id) => {
                    (String::from("OUTPUT PARAMETERS"), pin_id.clone())
                }
                super::TargetPin::NodePose(node_id, pin_id) => (node_id.clone(), pin_id.clone()),
                super::TargetPin::OutputPose => (String::from("OUTPUT POSE"), String::from("POSE")),
            };

            let color = match source_pin {
                super::SourcePin::NodeParameter(_, _) => "darkblue",
                super::SourcePin::InputParameter(_) => "darkblue",
                super::SourcePin::NodePose(_) => "chartreuse4",
                super::SourcePin::InputPose(_) => "chartreuse4",
            };

            writeln!(
                f,
                "\t\"{}\":\"{}\" -> \"{}\":\"{}\" [color={}",
                start_node, start_edge, end_node, end_edge, color
            )?;

            if let Some(context) = context.as_ref() {
                let time = context.get_prev_time(source_pin);

                writeln!(f, "label=\"{}\"", time)?;
            }

            writeln!(f, "];")?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}
