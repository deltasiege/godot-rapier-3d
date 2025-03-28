use bincode::{Decode, Encode};
use godot::prelude::*;
use rapier3d::parry::utils::hashmap::Entry;
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

/// Inserts an action into the given entry if it does not already exist
/// Returns the resulting Vec<Action> if the action was inserted
pub fn insert_action_if_allowed<T>(
    action: Action,
    entry: Entry<T, Vec<Action>>,
) -> Option<Vec<Action>> {
    let existing_actions = entry.or_insert(Vec::new());
    let already_has_op = existing_actions
        .iter()
        .any(|a| a.operation == action.operation);
    if !already_has_op {
        existing_actions.push(action);
        return Some(existing_actions.clone());
    } else {
        log::warn!(
            "Not inserting action {:?} because a matching action already exists",
            action
        );
        return None;
    }

    // TODO merge certain operations like move character or apply forces to RB,
    // should be allowed to apply multiple of them per tick
}
