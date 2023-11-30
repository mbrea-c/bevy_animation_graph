use super::{AnimationGraph, EdgeSpec, EdgeValue, TimeState, TimeUpdate};
use crate::{
    animation::HashMapJoinExt,
    core::{
        animation_node::{AnimationNode, NodeLike},
        frame::{BoneFrame, PoseFrame, ValueFrame},
        graph_context::GraphContext,
    },
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
        context: Option<&GraphContext>,
    ) -> std::io::Result<()>;

    fn preview_dot(&self, context: Option<&GraphContext>) -> std::io::Result<()> {
        let dir = std::env::temp_dir();
        let path = dir.join("bevy_animation_graph_dot.dot");

        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        self.to_dot(&mut writer, context)?;
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

    fn dot_to_tmp_file_and_open(&self, context: Option<&GraphContext>) -> std::io::Result<()> {
        self.dot_to_tmp_file(context)?;

        Command::new("zathura")
            .args(["/tmp/bevy_animation_graph_dot.dot.pdf"])
            .spawn()?;

        Ok(())
    }

    fn dot_to_tmp_file(&self, context: Option<&GraphContext>) -> std::io::Result<()> {
        let path = "/tmp/bevy_animation_graph_dot.dot";
        let pdf_path = "/tmp/bevy_animation_graph_dot.dot.pdf";
        let pdf_path_alt = "/tmp/bevy_animation_graph_dot.dot.pdf_alt";

        {
            let file = File::create(&path)?;
            let mut writer = BufWriter::new(file);
            self.to_dot(&mut writer, context)?;
        }

        {
            let pdf_file_alt = File::create(&pdf_path_alt)?;
            Command::new("dot")
                .args([path, "-Tpdf"])
                .stdout(pdf_file_alt)
                .status()?;

            std::fs::rename(pdf_path_alt, pdf_path)?;
        }

        Ok(())
    }
}

fn write_col(f: &mut impl std::io::Write, row: HashMap<String, EdgeSpec>) -> std::io::Result<()> {
    if !row.is_empty() {
        write!(f, "<TABLE BORDER=\"0\">")?;
        for (param_name, param_spec) in row.iter() {
            let icon = match param_spec {
                EdgeSpec::PoseFrame => String::from("🯅"),
                EdgeSpec::F32 => String::from("#"),
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

fn write_rows(
    f: &mut impl std::io::Write,
    left: HashMap<String, EdgeSpec>,
    right: HashMap<String, EdgeSpec>,
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

fn write_values<T: AsDotLabel>(
    f: &mut impl std::io::Write,
    row: &HashMap<String, T>,
) -> std::io::Result<()> {
    if !row.is_empty() {
        write!(f, "<TABLE BORDER=\"0\">")?;
        for (param_name, param_val) in row.iter() {
            write!(
                f,
                "<TR><TD>{}</TD><TD>{}</TD></TR>",
                param_name,
                param_val.as_dot_label()
            )?;
        }
        write!(f, "</TABLE>")?;
    }
    Ok(())
}

pub trait AsDotLabel {
    fn as_dot_label(&self) -> String;
}

impl AsDotLabel for EdgeValue {
    fn as_dot_label(&self) -> String {
        match self {
            EdgeValue::PoseFrame(p) => p.as_dot_label(),
            EdgeValue::F32(f) => format!("{:.3}", f),
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
        format!(
            "{:.3}-({:.3})-{:.3}",
            self.prev_timestamp, self.timestamp, self.next_timestamp
        )
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

fn write_debugdump(
    f: &mut impl std::io::Write,
    node: &AnimationNode,
    context: &GraphContext,
) -> std::io::Result<()> {
    write!(f, "<TR><TD COLSPAN=\"2\"><i>DebugDump</i></TD></TR>")?;
    if let Some(param_cache) = context
        .get_node_cache(&node.name)
        .map_or(None, |nc| nc.parameter_cache.as_ref())
    {
        write!(f, "<TR><TD COLSPAN=\"2\">Parameters</TD></TR>")?;
        write!(f, "<TR>")?;
        write!(f, "<TD>")?;
        write_values(f, &param_cache.upstream)?;
        write!(f, "</TD>")?;
        write!(f, "<TD>")?;
        write_values(f, &param_cache.downstream)?;
        write!(f, "</TD>")?;
        write!(f, "</TR>")?;
    }
    if let Some(duration_cache) = context
        .get_node_cache(&node.name)
        .map_or(None, |nc| nc.duration_cache.as_ref())
    {
        write!(f, "<TR><TD COLSPAN=\"2\">Durations</TD></TR>")?;
        write!(f, "<TR>")?;
        write!(f, "<TD>")?;
        write_values(f, &duration_cache.upstream)?;
        write!(f, "</TD>")?;
        write!(f, "<TD>")?;
        write!(f, "{:?}", duration_cache.downstream)?;
        write!(f, "</TD>")?;
        write!(f, "</TR>")?;
    }

    let tc = context.get_node_cache(&node.name).map(|nc| &nc.time_caches);
    if let Some(time_caches) = tc {
        if !time_caches.is_empty() {
            write!(f, "<TR><TD COLSPAN=\"2\">Time queries</TD></TR>")?;
            for (_, time_cache) in time_caches.iter() {
                write!(f, "<TR>")?;
                write!(f, "<TD>")?;
                write_values(f, &time_cache.upstream)?;
                write!(f, "</TD>")?;
                write!(f, "<TD>")?;
                write!(f, "{}", time_cache.downstream.as_dot_label())?;
                write!(f, "</TD>")?;
                write!(f, "</TR>")?;
            }
        }
    }
    let tdc = context
        .get_node_cache(&node.name)
        .map(|nc| &nc.time_dependent_caches);
    if let Some(time_dependent_caches) = tdc {
        if !time_dependent_caches.is_empty() {
            write!(f, "<TR><TD COLSPAN=\"2\">Time-dependent queries</TD></TR>")?;
            for (_, time_dependent_cache) in time_dependent_caches.iter() {
                write!(f, "<TR>")?;
                write!(f, "<TD>")?;
                write_values(f, &time_dependent_cache.upstream)?;
                write!(f, "</TD>")?;
                write!(f, "<TD>")?;
                write_values(f, &time_dependent_cache.downstream)?;
                write!(f, "</TD>")?;
                write!(f, "</TR>")?;
            }
        }
    }
    Ok(())
}

impl ToDot for AnimationGraph {
    fn to_dot(
        &self,
        f: &mut impl std::io::Write,
        context: Option<&GraphContext>,
    ) -> std::io::Result<()> {
        writeln!(f, "digraph {{")?;
        writeln!(f, "\trankdir=LR;")?;
        writeln!(f, "\tnode [style=rounded, shape=plain];")?;

        for (name, node) in self.nodes.iter() {
            write!(
                f,
                "\t\"{}\" [label=<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">",
                name
            )?;
            write!(
                f,
                "<TR><TD COLSPAN=\"2\"><B>{}</B><BR/><i>{}</i></TD></TR>",
                name,
                node.node_type_str()
            )?;

            let in_param = node.parameter_input_spec();
            let out_param = node.parameter_output_spec();

            let in_td = node.time_dependent_input_spec();
            let out_td = node.time_dependent_output_spec();

            write_rows(f, in_param, out_param)?;
            write_rows(f, in_td, out_td)?;

            if let Some(context) = context {
                write_debugdump(f, node, context)?;
            }

            writeln!(f, "</TABLE>>]")?;
        }

        writeln!(f, "OUTPUT [shape=cds];")?;
        writeln!(
            f,
            "\t\"{}\":\"{}\" -> OUTPUT;",
            self.out_node, self.out_edge
        )?;

        for ((end_node, end_edge), (start_node, start_edge)) in self.edges.iter() {
            let node = self.nodes.get(start_node).unwrap();
            let mut spec = node.parameter_output_spec();
            spec.fill_up(&node.time_dependent_output_spec(), &|v| v.clone());
            let tp = spec.get(start_edge).unwrap();
            let color = match tp {
                EdgeSpec::PoseFrame => "chartreuse4",
                EdgeSpec::F32 => "deeppink3",
            };

            writeln!(
                f,
                "\t\"{}\":\"{}\" -> \"{}\":\"{}\" [color={}];",
                start_node, start_edge, end_node, end_edge, color
            )?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}
