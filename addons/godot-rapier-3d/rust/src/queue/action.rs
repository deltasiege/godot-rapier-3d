use core::fmt::Debug;
use std::any::type_name;
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

use godot::obj::WithBaseField;

use crate::utils::HasCUID2Field;

use super::{Actionable, QueueName};

#[derive(Debug)]
pub struct Action {
    pub inner_cuid: String, // cuid2 of inner data
    pub inner_iid: i64,     // instance_id of inner data
    pub data: Actionable,
}

impl Action {
    pub fn step() -> Self {
        Self {
            inner_cuid: "step".to_string(),
            inner_iid: 0,
            data: Actionable::Step,
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Action: {} {}", self.inner_cuid, type_name::<Self>())
    }
}
impl Eq for Action {}
impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.inner_cuid == other.inner_cuid
    }
}
impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.data
                .cmp(&other.data)
                .then_with(|| self.inner_cuid.cmp(&other.inner_cuid)),
        )
    }
}
impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data
            .cmp(&other.data)
            .then_with(|| self.inner_cuid.cmp(&other.inner_cuid))
    }
}

pub trait CanDispatchActions: WithBaseField + HasCUID2Field {
    fn get_action(&self, data: Actionable) -> Action {
        Action {
            inner_cuid: self.get_cuid2(),
            inner_iid: self.base().instance_id().to_i64(),
            data,
        }
    }

    fn dispatch_action(&self, data: Actionable, queue_name: &QueueName) -> Result<(), String> {
        let action = self.get_action(data);
        let mut engine = crate::engine::get_engine()?;
        let mut bind = engine.bind_mut();
        bind.action_queue.add_action(action, queue_name);
        Ok(())
    }
}
