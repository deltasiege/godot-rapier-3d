use bincode::{Decode, Encode};
use godot::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(
    GodotConvert,
    Var,
    Export,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
#[godot(via = GString)]
pub enum Operation {
    AddNode,
    ConfigureNode,
    RemoveNode,
    MoveNode,
    TeleportNode,
}

#[derive(Debug, Clone)]
/// Struct stored in world buffer. Identical to Action but
/// cuid and handle are extracted from the node reference
/// to avoid re-fetching multiple times if the action is applied multiple times
pub struct Action {
    pub cuid: GString,
    pub handle: Option<(u32, u32)>,
    pub node: Gd<Node>,
    pub operation: Operation,
    pub data: Dictionary,
}

impl Action {
    pub fn new(
        cuid: GString,
        handle: Option<(u32, u32)>,
        node: Gd<Node>,
        operation: Operation,
        data: Dictionary,
    ) -> Self {
        Self {
            cuid,
            handle,
            node,
            operation,
            data,
        }
    }
}
