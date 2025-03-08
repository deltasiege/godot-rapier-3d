use crate::GR3DPhysicsState;
use godot::global::godot_print;
use rapier3d::pipeline::PhysicsPipeline;

pub fn step(state: &mut GR3DPhysicsState, physics_pipeline: &mut PhysicsPipeline) {
    for (h, col) in state.collider_set.iter() {
        let wrt = match col.position_wrt_parent() {
            Some(wrt) => Some(wrt.translation.vector),
            None => None,
        };
        godot_print!(
            "Collider: {:?} {:?} wrt: {:?}",
            h.into_raw_parts(),
            col.position().translation.vector,
            wrt,
        );
    }
    physics_pipeline.step(
        &state.gravity,
        &state.integration_parameters,
        &mut state.island_manager,
        &mut state.broad_phase,
        &mut state.narrow_phase,
        &mut state.rigid_body_set,
        &mut state.collider_set,
        &mut state.impulse_joint_set,
        &mut state.multibody_joint_set,
        &mut state.ccd_solver,
        Some(&mut state.query_pipeline),
        &(),
        &(),
    );
}
