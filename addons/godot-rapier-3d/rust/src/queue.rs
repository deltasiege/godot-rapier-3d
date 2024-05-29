use std::collections::HashMap;

use crate::lookups::Lookups;
use crate::queue::process::process_sync_action;
use crate::{GR3DPhysicsPipeline, GR3DPhysicsState, ObjectKind};
use process::{process_insert_action, process_parent_action, process_remove_action};

mod action;
mod actionable;
mod process;
pub use action::{Action, CanDispatchActions};
pub use actionable::Actionable;

use self::process::process_sim_action;

// Purpose of this struct is to store pending actions that need to be processed by the physics pipeline.
// All modifications of the pipeline need to be done in a deterministic order to ensure determinism of the simulation.
// Note: all actions are r2g (change rapier to match godot) since only Rapier modifications need to be queued.

pub struct ActionQueue {
    pub queues: HashMap<QueueName, Vec<Action>>,
}

impl ActionQueue {
    pub fn new() -> Self {
        Self {
            queues: HashMap::from([
                (QueueName::Insert, Vec::new()),
                (QueueName::Remove, Vec::new()),
                (QueueName::Parent, Vec::new()),
                (QueueName::Sync, Vec::new()),
                (QueueName::Sim, Vec::new()),
            ]),
        }
    }

    pub fn push_step_action(&mut self) {
        self.add_action(Action::step(), &QueueName::Sim);
    }

    pub fn add_action(&mut self, action: Action, queue: &QueueName) {
        let queue = self.queues.get_mut(queue).expect("Invalid queue name");
        queue.push(action);
    }

    pub fn _process(&mut self, pipeline: &mut GR3DPhysicsPipeline, lookups: &mut Lookups) {
        self.sort();
        let state = &mut pipeline.state;
        self.process_remove_actions(state, lookups).ok();
        self.process_insert_actions(state, lookups).ok();
        self.process_parent_actions(state, lookups).ok();
        self.process_sync_actions(state, lookups).ok();
        self.process_sim_actions(pipeline, lookups).ok();
    }

    fn sort(&mut self) {
        for (queue_name, queue) in self.queues.iter_mut() {
            if queue.is_empty() {
                continue;
            }
            if queue_name == &QueueName::Sim {
                continue;
            }
            log::trace!(
                "Sorting queue '{:?}': {:#?}",
                queue_name,
                debug_queue(queue)
            );
            queue.sort_unstable();
            log::trace!("Sorted queue '{:?}': {:#?}", queue_name, debug_queue(queue));
        }
    }

    fn process_insert_actions(
        &mut self,
        state: &mut GR3DPhysicsState,
        lookups: &mut Lookups,
    ) -> Result<(), String> {
        let queue = self
            .queues
            .get_mut(&QueueName::Insert)
            .ok_or("Insert queue is None")?;
        if queue.is_empty() {
            Ok(())
        } else {
            for action in queue.drain(..) {
                let cuid = action.inner_cuid.clone();
                let object_kind = ObjectKind::from(&action.data);
                log::trace!("[AQ]: Inserting {} {:?}", object_kind, cuid);
                match process_insert_action(action, state, lookups) {
                    Ok(_) => log::debug!("[AQ]: Inserted {} {:?}", object_kind, cuid),
                    Err(e) => {
                        log::error!("[AQ]: Error inserting {} {:?}: {}", object_kind, cuid, e);
                        continue;
                    }
                }
            }
            Ok(())
        }
    }

    fn process_remove_actions(
        &mut self,
        state: &mut GR3DPhysicsState,
        lookups: &mut Lookups,
    ) -> Result<(), String> {
        let queue = self
            .queues
            .get_mut(&QueueName::Remove)
            .ok_or("Remove queue is None")?;
        if queue.is_empty() {
            Ok(())
        } else {
            for action in queue.drain(..) {
                let cuid = action.inner_cuid.clone();
                let object_kind = ObjectKind::from(&action.data);
                log::trace!("[AQ]: Removing {} {:?}", object_kind, cuid);
                match process_remove_action(action, state, lookups) {
                    Ok(_) => log::debug!("[AQ]: Removed {} {:?}", object_kind, cuid),
                    Err(e) => {
                        log::error!("[AQ]: Error removing {} {:?}: {}", object_kind, cuid, e);
                        continue;
                    }
                }
            }
            Ok(())
        }
    }

    fn process_parent_actions(
        &mut self,
        state: &mut GR3DPhysicsState,
        lookups: &Lookups,
    ) -> Result<(), String> {
        let queue = self
            .queues
            .get_mut(&QueueName::Parent)
            .ok_or("Parent queue is None")?;
        if queue.is_empty() {
            Ok(())
        } else {
            for action in queue.drain(..) {
                let cuid = action.inner_cuid.clone();
                let object_kind = ObjectKind::from(&action.data);
                log::trace!("[AQ]: Parenting {} {:?}", object_kind, cuid);
                match process_parent_action(action, state, lookups) {
                    Ok(has_parent) => {
                        let op = if has_parent { "Parented" } else { "Unparented" };
                        log::debug!("[AQ]: {} {} {:?}", op, object_kind, cuid)
                    }
                    Err(e) => {
                        log::error!("[AQ]: Error parenting {} {:?}: {}", object_kind, cuid, e);
                        continue;
                    }
                }
            }
            Ok(())
        }
    }

    fn process_sync_actions(
        &mut self,
        state: &mut GR3DPhysicsState,
        lookups: &mut Lookups,
    ) -> Result<(), String> {
        let queue = self
            .queues
            .get_mut(&QueueName::Sync)
            .ok_or("Sync queue is None")?;
        if queue.is_empty() {
            Ok(())
        } else {
            for action in queue.drain(..) {
                let cuid = action.inner_cuid.clone();
                let object_kind = ObjectKind::from(&action.data);
                log::trace!("[AQ]: Syncing {} {:?}", object_kind, cuid);
                match process_sync_action(action, state, lookups) {
                    Ok(_) => log::trace!("[AQ]: Synced {} {:?}", object_kind, cuid),
                    Err(e) => {
                        log::error!("[AQ]: Error syncing {} {:?}: {}", object_kind, cuid, e);
                        continue;
                    }
                }
            }
            Ok(())
        }
    }

    fn process_sim_actions(
        &mut self,
        pipeline: &mut GR3DPhysicsPipeline,
        lookups: &Lookups,
    ) -> Result<(), String> {
        let queue = self
            .queues
            .get_mut(&QueueName::Sim)
            .ok_or("Sim queue is None")?;
        if queue.is_empty() {
            Ok(())
        } else {
            for action in queue.drain(..) {
                log::trace!("[AQ]: Simulating");
                match process_sim_action(action, pipeline, lookups) {
                    Ok(_) => log::trace!("[AQ]: Simulated"),
                    Err(e) => {
                        log::error!("[AQ]: Error while simulating: {}", e);
                        continue;
                    }
                }
            }
            Ok(())
        }
    }
}

#[derive(Eq, PartialEq, Debug, Hash)]
pub enum QueueName {
    Insert,
    Remove,
    Parent,
    Sync,
    Sim,
}

fn debug_queue(actions: &Vec<Action>) -> Vec<String> {
    actions
        .iter()
        .map(|a| {
            format!(
                "{}, {}, {}",
                ObjectKind::from(&a.data),
                a.inner_cuid.clone(),
                a.data
            )
        })
        .collect()
}
