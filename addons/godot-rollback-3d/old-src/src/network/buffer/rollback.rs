/// Rolls this world back to the given timestep and:
/// - optionally adds the given actions to the buffer
/// - optionally applies the given snapshot physics state
/// - re-simulates the world back to the current timestep with changes applied
pub fn corrective_rollback(
    &mut self,
    timestep_id: usize,
    actions_to_add: Option<Vec<Action>>, // TODO do I need to support adding actions at different timesteps during a single rollback? probably
    snapshot: Option<DeserializedPhysicsSnapshot>,
) {
    if let Some(_target_step) = self.buffer.get_step_mut(timestep_id) {
        let current_timestep = self.state.timestep_id.clone();
        let steps_to_resim = current_timestep - timestep_id;

        match true {
            _ if timestep_id >= current_timestep => {
                log::error!("Cannot rollback to a future timestep: {}", timestep_id);
            }
            _ if steps_to_resim == 0 => {
                log::warn!("Corrective rollback to same timestep. No action taken.");
            }
            _ if steps_to_resim > self.buffer.max_len => {
                log::error!(
                    "Cannot rollback more than the buffer length: {}",
                    self.buffer.max_len
                );
            }
            _ => {
                if let Some(actions_to_add) = actions_to_add {
                    for action in actions_to_add {
                        self.buffer.insert_actions(vec![action], timestep_id);
                    }
                }

                if let Some(snapshot) = snapshot {
                    restore_snapshot(self, snapshot, true);
                }

                self.state.timestep_id = timestep_id;
                self.buffer.mark_physics_stale_after(timestep_id);

                for _ in 0..steps_to_resim {
                    self.step();
                }
            }
        }
    }
}
