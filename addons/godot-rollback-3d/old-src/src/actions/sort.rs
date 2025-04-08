use godot::prelude::*;
use std::cmp::Ordering;

use crate::{
    actions::{Action, Operation},
    utils::extract_from_dict,
};

/// Sort actions by CUID and then by operation and then by magnitude of movement
impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.cuid != other.cuid {
            self.cuid.cmp(&other.cuid)
        } else if self.operation != other.operation {
            self.operation.cmp(&other.operation)
        } else if self.operation == Operation::MoveNode && other.operation == Operation::MoveNode {
            if !self.data.contains_key("movement") || !other.data.contains_key("movement") {
                return Ordering::Equal;
            }
            let self_movement = extract_from_dict::<Vector3>(&self.data, "movement", false);
            let other_movement = extract_from_dict::<Vector3>(&other.data, "movement", false);
            if let Some(self_movement) = self_movement {
                if let Some(other_movement) = other_movement {
                    self_movement
                        .length()
                        .partial_cmp(&other_movement.length())
                        .unwrap()
                } else {
                    Ordering::Greater
                }
            } else {
                Ordering::Less
            }
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Action {}
impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.cuid == other.cuid && self.operation == other.operation && self.data == other.data
    }
}
