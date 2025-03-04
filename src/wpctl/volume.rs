use crate::wpctl::node::get_default_node;
use crate::wpctl::volume::OpType::Toggle;
use crate::wpctl::WPCTL_EXEC;
use crate::{notify, NodeType};
use std::process::Command;

const DEFAULT_MODIFY_STEP: u32 = 5;
const DEFAULT_SINK_SPECIFIER: &str = "@DEFAULT_AUDIO_SINK@";
const DEFAULT_SOURCE_SPECIFIER: &str = "@DEFAULT_AUDIO_SOURCE@";
const MAXIMUM_VOLUME: f32 = 1.5;
const MINIMUM_MODIFY_STEP: f32 = 0.01;
const MUTED_SUFFIX: &str = "[MUTED]";
const NOTIFICATION_NODE_MAX_LENGTH: usize = 24;

pub struct VolumeOp {
    op_type: OpType,
    step: Option<u32>,
    node_type: NodeType,
}

impl VolumeOp {
    pub fn new(change_type: OpType, step: Option<u32>, node_type: NodeType) -> Self {
        VolumeOp {
            op_type: change_type,
            step,
            node_type,
        }
    }
}

#[derive(Debug)]
pub enum OpType {
    Dec,
    Get,
    Inc,
    Set { value: u32 },
    Show,
    Toggle,
}

fn get_source_specifier(node_type: &NodeType) -> String {
    match node_type.clone() {
        NodeType::Sink => DEFAULT_SINK_SPECIFIER.to_string(),
        NodeType::Source => DEFAULT_SOURCE_SPECIFIER.to_string(),
    }
}

fn lookup(node_type: &NodeType) -> f32 {
    let mut cmd = Command::new(WPCTL_EXEC);
    let source_specifier = get_source_specifier(node_type);
    cmd.args(["get-volume", source_specifier.as_str()]);
    let out = cmd.output().expect("error getting volume");
    let vol_out = String::from_utf8(out.stdout).expect("error parsing cmd output");
    let vol_out_trim = vol_out.trim();
    if vol_out_trim.ends_with(MUTED_SUFFIX) {
        return 0.0;
    }

    let vol = vol_out_trim
        .split(" ")
        .nth(1)
        .expect("Error getting volume field");
    let vol_f: f32 = vol.parse().expect("error parsing volume");
    vol_f
}

fn modify(step: Option<u32>, sign: Option<&str>, node_type: &NodeType) {
    let mut cmd = Command::new(WPCTL_EXEC);

    let modify_step = step.unwrap_or(DEFAULT_MODIFY_STEP);
    let mut modify_volume = f32::max(modify_step as f32 / 100.0, MINIMUM_MODIFY_STEP).to_string();
    let max_vol = MAXIMUM_VOLUME.to_string();

    if sign.is_some() {
        modify_volume.push_str(sign.unwrap());
    }

    let node_specifier = get_source_specifier(&node_type);

    cmd.args([
        "set-volume",
        "-l",
        max_vol.as_str(),
        node_specifier.as_str(),
        format!("{modify_volume}").as_str(),
    ]);
    cmd.status().expect("error setting volume");
}

fn modify_rel(step: Option<u32>, sign: &str, node_type: &NodeType) {
    modify(step, Some(sign), node_type);
}

fn modify_set(value: u32, node_type: &NodeType) {
    modify(Some(value), None, node_type);
}

fn toggle(specifier: String) {
    let mut cmd = Command::new(WPCTL_EXEC);

    cmd.args(["set-mute", specifier.as_str(), "toggle"]);
    cmd.status().expect("error toggling volume");
}

fn node_type_to_str(node_type: &NodeType) -> String {
    match node_type {
        NodeType::Sink => "sink".to_string(),
        NodeType::Source => "source".to_string(),
    }
}

fn truncate_node_name(node: String) -> String {
    let mut words = node.split(" ").collect::<Vec<_>>();
    let mut cur_name = "".to_string();
    loop {
        if words.len() == 0 {
            break;
        }

        let next = words.get(0).cloned().unwrap();
        words.remove(0);
        if cur_name.len() == 0 {
            cur_name.push_str(next);
            continue;
        }

        if cur_name.len() + 1 + next.len() < NOTIFICATION_NODE_MAX_LENGTH {
            cur_name.push_str(" ");
            cur_name.push_str(next);
        } else {
            break;
        }
    }
    cur_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(
            truncate_node_name("foo bar baz qux fred barney".to_string()),
            "foo bar baz qux fred".to_string()
        );
        assert_eq!(
            truncate_node_name("foobarbazquxfredbarneywaldo".to_string()),
            "foobarbazquxfredbarneywaldo".to_string()
        );
    }
}

fn notify(volume: f32, node_type: &NodeType) {
    let node = get_default_node(node_type_to_str(node_type));
    if node.is_none() {
        notify::message(
            format!("Cannot determine default {}", node_type_to_str(node_type)).as_str(),
        );
        return;
    }
    let truncated = truncate_node_name(node.unwrap());
    notify::volume(volume, truncated);
}

pub fn apply(change: VolumeOp) {
    let node_type = change.node_type.clone();
    let old_volume = lookup(&node_type);
    match change.op_type {
        dec_or_inc @ (OpType::Dec | OpType::Inc) => {
            let sign = match dec_or_inc {
                OpType::Dec => "-",
                OpType::Inc => "+",
                _ => unreachable!("Unexpected volume change type {:?}", dec_or_inc),
            };
            modify_rel(change.step, sign, &node_type);
        }
        OpType::Get => {
            println!("volume: {}", old_volume);
            return;
        }
        OpType::Set { value } => modify_set(value, &node_type),
        OpType::Show => {
            notify(old_volume, &node_type);
            return;
        }
        Toggle => toggle(get_source_specifier(&node_type)),
    };

    let new_volume = lookup(&change.node_type);
    if old_volume != new_volume || new_volume == 0.0 {
        notify(new_volume, &node_type);
    }
}
