use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum Node {
    Str(String),
    Arr(Vec<Node>),
    Obj(BTreeMap<String, Node>),
}

impl Node {
    pub fn as_obj_mut(&mut self) -> Option<&mut BTreeMap<String, Node>> {
        match self {
            Node::Obj(m) => Some(m),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RunOptions {
    pub json: bool,
    pub markdown: bool,
    pub toml: bool,
    pub show_not_found: bool,
    pub duplicates: bool,
    pub full_tree: bool,
}
