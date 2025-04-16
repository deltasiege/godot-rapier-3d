use godot::prelude::*;
use rapier3d::control::{KinematicCharacterController, PdController};
use rapier3d::parry::either::Either::{Left, Right};
use rapier3d::prelude::{AxesMask, QueryFilter, RigidBodyHandle, RigidBodyVelocity};

use crate::types::RollbackNodeClass;
use crate::utils::vector_to_rapier;
use crate::world::{NodeOperation, PhysicsState};
use crate::{impl_trait_for_nodes, nodes::*};

pub trait Controllable: HasNodeData + RollbackNode {
    fn on_move_by_amount(&self, amount: Vector3) {
        if amount == Vector3::ZERO {
            return;
        }
        self.ingest_modify_action(NodeOperation::MoveByAmount, &[amount.to_variant()]);
    }

    fn on_teleport_to_position(&self, position: Vector3) {
        self.ingest_modify_action(NodeOperation::TeleportToPosition, &[position.to_variant()]);
    }
}

impl_trait_for_nodes!(
    Controllable,
    {},
    RollbackKinematicCharacter3D,
    RollbackPIDCharacter3D
);

/// Move a node by a given amount in the Rapier world.
pub fn rapier_move_node(node_data: &NodeData, amount: Option<Variant>, physics: &mut PhysicsState) {
    let amount = amount
        .and_then(|v| v.try_to::<Vector3>().ok())
        .unwrap_or_default();

    if amount == Vector3::ZERO {
        return;
    }

    let handle = match node_data.get_rapier_handle() {
        Left(handle) => handle,
        _ => {
            log::error!("Cannot move RollbackCollider3D");
            return;
        }
    };

    match node_data.blueprint.class {
        RollbackNodeClass::RollbackKinematicCharacter3D => {
            rapier_move_kinematic_character(
                handle,
                node_data
                    .blueprint
                    .rapier_kinematic_controller
                    .clone()
                    .unwrap(),
                amount,
                physics,
            );
        }
        RollbackNodeClass::RollbackPIDCharacter3D | RollbackNodeClass::RollbackRigidBody3D => {
            match &node_data.blueprint.rapier_pid_controller {
                Some(controller_settings) => {
                    rapier_move_rigidbody(handle, controller_settings, amount, physics);
                }
                None => {
                    log::error!(
                        "Cannot move node '{}' - no PID controller found.",
                        node_data.blueprint.class
                    );
                    return;
                }
            }
        }
        _ => log::error!("Cannot move node '{}'", node_data.blueprint.class),
    }

    match node_data.get_rapier_handle() {
        Left(handle) => {
            if !physics.bodies.contains(handle) {
                return;
            }
        }
        Right(handle) => {
            if !physics.colliders.contains(handle) {
                return;
            }
        }
    }
}

/// Teleport a node to a given position in the Rapier world.
pub fn rapier_teleport_node(
    node_data: &NodeData,
    position: Option<Variant>,
    physics: &mut PhysicsState,
) {
    let position = match position.clone().and_then(|v| v.try_to::<Vector3>().ok()) {
        Some(position) => position,
        None => {
            log::error!("Cannot teleport node - invalid position: {:?}.", position);
            return;
        }
    };

    let name = node_data.blueprint.get_node_name();
    match node_data.blueprint.class {
        RollbackNodeClass::RollbackKinematicCharacter3D
        | RollbackNodeClass::RollbackPIDCharacter3D
        | RollbackNodeClass::RollbackRigidBody3D => match node_data.get_rapier_handle().left() {
            Some(handle) => {
                if !physics.bodies.contains(handle) {
                    log::error!(
                        "Cannot teleport node '{}' - handle not found in physics state.",
                        name
                    );
                    return;
                }

                let body = &mut physics.bodies[handle];
                body.set_next_kinematic_translation(vector_to_rapier(position));
            }
            None => {
                log::error!("Cannot teleport node '{}' - invalid rapier handle.", name);
                return;
            }
        },
        _ => log::error!("Cannot teleport '{}' nodes", node_data.blueprint.class),
    }
}

/// Move a kinematic character controller by a given amount in the Rapier world.
fn rapier_move_kinematic_character(
    handle: RigidBodyHandle,
    controller: KinematicCharacterController,
    amount: Vector3,
    physics: &mut PhysicsState,
) {
    if !physics.bodies.contains(handle) {
        return;
    }

    let body = &mut physics.bodies[handle];
    let collider = &physics.colliders[body.colliders()[0]];
    let mass = body.mass();

    let mut collisions = vec![];
    let movement = controller.move_shape(
        physics.integration_parameters.dt,
        &physics.bodies,
        &physics.colliders,
        &physics.query_pipeline,
        collider.shape(),
        collider.position(),
        vector_to_rapier(amount),
        QueryFilter::new().exclude_rigid_body(handle),
        |c| collisions.push(c),
    );

    // Apply impulses to other rigidbodies that were contacted
    controller.solve_character_collision_impulses(
        physics.integration_parameters.dt,
        &mut physics.bodies,
        &physics.colliders,
        &physics.query_pipeline,
        collider.shape(),
        mass,
        &*collisions,
        QueryFilter::new().exclude_rigid_body(handle),
    );

    let body = &mut physics.bodies[handle];
    let pose = body.next_position();
    body.set_next_kinematic_translation(pose.translation.vector + movement.translation);

    // TODO somehow set these where the Godot node can access them
    // char.last_movement = Some(movement);
    // char.last_collisions = collisions;
}

/// Move a rigid body by a given amount in the Rapier world using a PID controller.
fn rapier_move_rigidbody(
    handle: RigidBodyHandle,
    controller_settings: &PDControllerSettings,
    amount: Vector3,
    physics: &mut PhysicsState,
) {
    if !physics.bodies.contains(handle) {
        return;
    }

    let body = &mut physics.bodies[handle];

    let mut axes = AxesMask::ANG_X | AxesMask::ANG_Y | AxesMask::ANG_Z;
    let mvmt = vector_to_rapier(amount);

    if mvmt.norm() != 0.0 {
        axes |= if amount.y == 0.0 {
            AxesMask::LIN_X | AxesMask::LIN_Z
        } else {
            AxesMask::LIN_X | AxesMask::LIN_Z | AxesMask::LIN_Y
        }
    };

    let controller = PdController::new(controller_settings.kp, controller_settings.kd, axes);
    let corrective_vel = controller.rigid_body_correction(
        body,
        (body.translation() + mvmt).into(),
        RigidBodyVelocity::zero(),
    );

    body.set_vels(*body.vels() + corrective_vel, true);
}
