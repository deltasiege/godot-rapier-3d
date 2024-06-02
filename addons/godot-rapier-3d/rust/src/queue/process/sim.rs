use crate::GR3DPhysicsPipeline;

pub fn step(pipeline: &mut GR3DPhysicsPipeline) {
    pipeline.physics_pipeline.step(
        &pipeline.state.gravity,
        &pipeline.state.integration_parameters,
        &mut pipeline.state.island_manager,
        &mut pipeline.state.broad_phase,
        &mut pipeline.state.narrow_phase,
        &mut pipeline.state.rigid_body_set,
        &mut pipeline.state.collider_set,
        &mut pipeline.state.impulse_joint_set,
        &mut pipeline.state.multibody_joint_set,
        &mut pipeline.state.ccd_solver,
        Some(&mut pipeline.state.query_pipeline),
        &pipeline.physics_hooks,
        &pipeline.event_handler,
    );
}
