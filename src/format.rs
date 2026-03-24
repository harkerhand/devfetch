use std::collections::BTreeMap;

use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::model::{Node, RunOptions};

fn clean_node(node: &Node, show_not_found: bool) -> Option<Node> {
    match node {
        Node::Str(s) => {
            if s == "N/A" || (!show_not_found && s == "Not Found") {
                None
            } else {
                Some(Node::Str(s.clone()))
            }
        }
        Node::Arr(a) => {
            let vals: Vec<Node> = a
                .iter()
                .filter_map(|n| clean_node(n, show_not_found))
                .collect();
            if vals.is_empty() {
                None
            } else {
                Some(Node::Arr(vals))
            }
        }
        Node::Obj(obj) => {
            let mut m = BTreeMap::new();
            for (k, v) in obj {
                if let Some(c) = clean_node(v, show_not_found) {
                    m.insert(k.clone(), c);
                }
            }
            if m.is_empty() {
                None
            } else {
                Some(Node::Obj(m))
            }
        }
    }
}

fn serialize_arrays(node: Node) -> Node {
    match node {
        Node::Arr(arr) => {
            let vals: Vec<String> = arr
                .into_iter()
                .filter_map(|n| match n {
                    Node::Str(s) => Some(s),
                    _ => None,
                })
                .collect();
            if vals.is_empty() {
                Node::Str("None".to_string())
            } else {
                Node::Str(vals.join(", "))
            }
        }
        Node::Obj(obj) => {
            let mut m = BTreeMap::new();
            for (k, v) in obj {
                m.insert(k, serialize_arrays(v));
            }
            Node::Obj(m)
        }
        other => other,
    }
}

fn serialize_version_paths(node: Node) -> Node {
    match node {
        Node::Obj(obj) => {
            if let Some(Node::Str(version)) = obj.get("version") {
                let path = match obj.get("path") {
                    Some(Node::Str(p)) => Some(p.clone()),
                    _ => None,
                };
                return Node::Str(match path {
                    Some(p) => format!("{version} - {p}"),
                    None => version.clone(),
                });
            }
            let mut out = BTreeMap::new();
            for (k, v) in obj {
                out.insert(k, serialize_version_paths(v));
            }
            Node::Obj(out)
        }
        other => other,
    }
}

fn node_to_json_value(node: Node) -> JsonValue {
    match node {
        Node::Str(s) => JsonValue::String(s),
        Node::Arr(arr) => JsonValue::Array(arr.into_iter().map(node_to_json_value).collect()),
        Node::Obj(obj) => {
            let mut map = JsonMap::new();
            for (k, v) in obj {
                map.insert(k, node_to_json_value(v));
            }
            JsonValue::Object(map)
        }
    }
}

const YAML_INDENT: usize = 4;

fn yaml_to_markdown(yaml: &str) -> String {
    let mut out = String::new();
    for line in yaml.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();
        if trimmed.ends_with(':') {
            let level = indent / YAML_INDENT + 1;
            out.push_str(&format!("{} {}\n", "#".repeat(level), trimmed));
        } else {
            out.push_str(&format!(" - {}\n", trimmed));
        }
    }
    out
}

fn to_pretty_json(value: &JsonValue) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
}

fn to_indented_yaml(value: &JsonValue) -> String {
    let raw = serde_yaml::to_string(value).unwrap_or_else(|_| "{}\n".to_string());
    let raw = raw.strip_prefix("---\n").unwrap_or(&raw);

    let mut out = String::new();
    out.push('\n');
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        out.push_str(&" ".repeat(YAML_INDENT));
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');
    out
}

fn to_pretty_toml(value: &JsonValue) -> String {
    toml::to_string_pretty(value).unwrap_or_else(|_| String::new())
}

pub fn render(report: Node, options: &RunOptions) -> String {
    let cleaned = clean_node(&report, options.show_not_found).unwrap_or(Node::Obj(BTreeMap::new()));

    if options.json {
        let value = node_to_json_value(cleaned);
        let body = to_pretty_json(&value);
        return format!("{body}\n");
    }

    let normalized = serialize_version_paths(serialize_arrays(cleaned));
    let value = node_to_json_value(normalized);

    if options.toml {
        let body = to_pretty_toml(&value);
        return format!("{body}\n");
    }

    let yaml = to_indented_yaml(&value);

    if options.markdown {
        return yaml_to_markdown(&yaml);
    }

    yaml
}
