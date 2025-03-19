use crate::nodes::{IRapierObject, Identifiable};
use crate::nodes::{
    RapierArea3D, RapierCollisionShape3D, RapierKinematicCharacter3D, RapierPIDCharacter3D,
    RapierRigidBody3D, RapierStaticBody3D,
};
use crate::utils::{transform_to_isometry, vector_to_point, vector_to_rapier};
use crate::World;
use godot::classes::{
    BoxShape3D, CapsuleShape3D, ConcavePolygonShape3D, CylinderShape3D, SphereShape3D,
};
use godot::prelude::*;
use rapier3d::prelude::*;

const SUPPORTED_SHAPES: &str = "Only primitives and ConcavePolygonShape3D are supported";

pub fn add_nodes_to_world(nodes: Array<Gd<Node3D>>, world: &mut World) {
    for node in nodes.iter_shared() {
        let transform = node.get_global_transform();
        let class = node.get_class().to_string();
        match class.as_str() {
            "RapierArea3D" => {
                let casted = node.cast::<RapierArea3D>();
                insert_area_children(&casted, world);
            }
            "RapierKinematicCharacter3D" => {
                let rb = RigidBodyBuilder::kinematic_position_based()
                    .position(transform_to_isometry(transform))
                    .ccd_enabled(true);

                let mut casted = node.cast::<RapierKinematicCharacter3D>();
                insert_rb_with_children(rb, &mut casted, world);
            }
            "RapierPIDCharacter3D" => {
                let rb = RigidBodyBuilder::dynamic().position(transform_to_isometry(transform));
                let mut casted = node.cast::<RapierPIDCharacter3D>();
                insert_rb_with_children(rb, &mut casted, world);
            }
            "RapierRigidBody3D" => {
                let rb = RigidBodyBuilder::dynamic().position(transform_to_isometry(transform));
                let mut casted = node.cast::<RapierRigidBody3D>();
                insert_rb_with_children(rb, &mut casted, world);
            }
            "RapierStaticBody3D" => {
                let rb = RigidBodyBuilder::fixed().position(transform_to_isometry(transform));
                let mut casted = node.cast::<RapierStaticBody3D>();
                insert_rb_with_children(rb, &mut casted, world);
            }
            "RapierCollisionShape3D" => (), // Ignore colliders - they are inserted at the same time as the parent rigid body
            _ => {
                log::error!("Unknown object type: {}", &class);
            }
        }
    }
}

fn insert_area_children(node: &Node3D, world: &mut World) {
    let children = node
        .find_children_ex("*")
        .type_("RapierCollisionShape3D")
        .recursive(true)
        .owned(false)
        .done();

    match children.len() {
        0 => {
            log::error!(
                "'{}' must have a child RapierCollisionShape3D",
                node.get_name()
            );
        }
        _ => {
            for child in children.iter_shared() {
                let mut casted = child.cast::<RapierCollisionShape3D>();
                insert_collider(&mut casted, None, world, true);
            }
        }
    }
}

fn insert_rb_with_children(
    rb: impl Into<RigidBody>,
    node: &mut Gd<impl IRapierObject>,
    world: &mut World,
) {
    let bodies = &mut world.physics.bodies;

    let children = node
        .find_children_ex("*")
        .type_("RapierCollisionShape3D")
        .recursive(true)
        .owned(false)
        .done();

    match children.len() {
        0 => {
            log::error!(
                "'{}' must have a child RapierCollisionShape3D",
                node.get_name()
            );
        }
        _ => {
            let parent_handle = bodies.insert(rb);

            for child in children.iter_shared() {
                let mut casted = child.cast::<RapierCollisionShape3D>();
                insert_collider(&mut casted, Some(parent_handle), world, false);
            }

            let node_uid = node.bind().get_cuid();
            let raw_handle = parent_handle.into_raw_parts();
            world.physics.lookup_table.insert(node_uid, raw_handle);
            node.bind_mut().set_handle_raw(raw_handle);
        }
    }
}

fn insert_collider(
    node: &mut Gd<RapierCollisionShape3D>,
    parent: Option<RigidBodyHandle>,
    world: &mut World,
    sensor: bool, // TODO - could be exposed to godot by reading from node directly in here
) {
    let lookup_table = &mut world.physics.lookup_table;

    if let Some(collider) = shape_to_collider(node) {
        let is_exp = is_expensive(&collider);

        let built = collider.sensor(sensor).build();
        let handle = match parent {
            Some(parent) => &mut world.physics.colliders.insert_with_parent(
                built,
                parent,
                &mut world.physics.bodies,
            ),
            None => &mut world.physics.colliders.insert(built),
        };

        let raw_handle = handle.into_raw_parts();
        let node_uid = node.bind().get_cuid();
        lookup_table.insert(node_uid, raw_handle);
        if !is_exp {
            lookup_table.insert_snapshot_collider(raw_handle);
        }
        node.bind_mut().set_handle_raw(raw_handle);
    }
}

fn shape_to_collider(node: &Gd<RapierCollisionShape3D>) -> Option<ColliderBuilder> {
    let shape = node.bind().get_shape();
    let transform = node.get_transform(); // Get LOCAL transform to maintain relative position to parent
    match shape {
        Some(shape) => match shape.get_class().to_string().as_str() {
            "SphereShape3D" => {
                let casted = shape.cast::<SphereShape3D>();
                Some(
                    ColliderBuilder::ball(casted.get_radius())
                        .position(transform_to_isometry(transform)),
                )
            }
            "BoxShape3D" => {
                let casted = shape.cast::<BoxShape3D>();
                Some(
                    ColliderBuilder::cuboid(
                        casted.get_size().x / 2.0,
                        casted.get_size().y / 2.0,
                        casted.get_size().z / 2.0,
                    )
                    .position(transform_to_isometry(transform)),
                )
            }
            "CapsuleShape3D" => {
                let casted = shape.cast::<CapsuleShape3D>();
                Some(
                    ColliderBuilder::capsule_y(casted.get_height() / 4.0, casted.get_radius())
                        .position(transform_to_isometry(transform)),
                )
            }
            "CylinderShape3D" => {
                let casted = shape.cast::<CylinderShape3D>();
                Some(
                    ColliderBuilder::cylinder(casted.get_height() / 2.0, casted.get_radius())
                        .position(transform_to_isometry(transform)),
                )
            }
            "ConcavePolygonShape3D" => {
                let casted = shape.cast::<ConcavePolygonShape3D>();

                let faces = casted.get_faces().to_vec();
                let mut vertices: Vec<Point<f32>> = vec![];
                let mut indices: Vec<[u32; 3]> = vec![];

                for (idx, vert) in faces.iter().enumerate() {
                    vertices.push(vector_to_point(&vector_to_rapier(vert.clone())));
                    if idx % 3 == 0 {
                        indices.push([idx as u32, (idx + 1) as u32, (idx + 2) as u32]);
                    }
                }

                let trimesh =
                    ColliderBuilder::trimesh_with_flags(vertices, indices, TriMeshFlags::empty());
                match trimesh {
                    Ok(trimesh) => {
                        if is_expensive(&trimesh) {
                            if let Some(tree) = node.get_tree() {
                                if tree.is_debugging_collisions_hint() {
                                    log::warn!("Debug rendering of '{:?}' collider will be skipped because it has more than {} vertices", node.get_name(), crate::config::DEBUG_MAX_VERTEX_COUNT);
                                }
                            }
                        }

                        Some(trimesh.position(transform_to_isometry(transform)))
                    }
                    Err(_) => {
                        log::error!("Failed to create trimesh collider from collision shape");
                        None
                    }
                }
            }
            _ => {
                log::error!("Unknown shape class: {}", shape.get_class().to_string());
                None
            }
        },
        None => {
            log::error!(
                "Missing shape on: '{}'. {}",
                node.get_name(),
                SUPPORTED_SHAPES
            );
            None
        }
    }
}

pub fn remove_nodes_from_world(nodes: Array<Gd<Node3D>>, world: &mut World) {
    for node in nodes.iter_shared() {
        let class = node.get_class().to_string();
        match class.as_str() {
            "RapierArea3D" => {
                let casted = node.cast::<RapierCollisionShape3D>();
                remove_collider_node_if_exists(&casted, world);
            }
            "RapierKinematicCharacter3D" => {
                let casted = node.cast::<RapierKinematicCharacter3D>();
                remove_body(&casted, world);
            }
            "RapierRigidBody3D" => {
                let casted = node.cast::<RapierRigidBody3D>();
                remove_body(&casted, world);
            }
            "RapierStaticBody3D" => {
                let casted = node.cast::<RapierStaticBody3D>();
                remove_body(&casted, world);
            }
            "RapierCollisionShape3D" => {
                let casted = node.cast::<RapierCollisionShape3D>();
                remove_collider_node_if_exists(&casted, world);
            }
            _ => {
                log::error!("Unknown object type: {}", &class);
            }
        }
    }
}

/// Returns true if the collider is expensive to render/serialize
/// and should be excluded from snapshots and debug rendering
fn is_expensive(builder: &ColliderBuilder) -> bool {
    match builder.shape.as_trimesh() {
        Some(trimesh) => {
            let vertices = trimesh.vertices().len();
            vertices > crate::config::DEBUG_MAX_VERTEX_COUNT
        }
        None => false,
    }
}

/// Removes the given RapierCollisionShape3D from cheap or expensive colliders if it exists in either set
fn remove_collider_node_if_exists(node: &Gd<RapierCollisionShape3D>, world: &mut World) {
    let node_uid = node.bind().get_cuid();
    if let Some(raw_handle) = world.physics.lookup_table.remove_by_uid(&node_uid.into()) {
        remove_collider_if_exists(&raw_handle, world);
    }
}

/// Removes the given collider handle from all lookup tables and collider set
pub fn remove_collider_if_exists(raw_handle: &(u32, u32), world: &mut World) {
    world.physics.lookup_table.remove_by_handle(raw_handle);
    world
        .physics
        .lookup_table
        .remove_snapshot_collider(raw_handle);
    let handle = ColliderHandle::from_raw_parts(raw_handle.0, raw_handle.1);
    if world.physics.colliders.contains(handle) {
        world.physics.colliders.remove(
            handle,
            &mut world.physics.islands,
            &mut world.physics.bodies,
            false,
        );
    }
}

/// Removes the given rigid body from the world
fn remove_body(node: &Gd<impl IRapierObject>, world: &mut World) {
    let node_uid = node.bind().get_cuid();
    if let Some(handle) = world.physics.lookup_table.remove_by_uid(&node_uid) {
        world.physics.bodies.remove(
            RigidBodyHandle::from_raw_parts(handle.0, handle.1),
            &mut world.physics.islands,
            &mut world.physics.colliders,
            &mut world.physics.impulse_joints,
            &mut world.physics.multibody_joints,
            false,
        );
    }
}
