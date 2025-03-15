use crate::nodes::Identifiable;
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
                let handle = insert_rb_with_children(rb, &node, world);

                match handle {
                    Some(handle) => {
                        let mut casted = node.cast::<RapierKinematicCharacter3D>();
                        let node_uid = casted.bind().get_cuid();
                        let raw_handle = handle.into_raw_parts();
                        world.lookup_table.insert(node_uid, raw_handle);
                        casted.bind_mut().set_handle_raw(raw_handle);
                    }
                    None => (),
                };
            }
            "RapierPIDCharacter3D" => {
                let rb = RigidBodyBuilder::dynamic().position(transform_to_isometry(transform));
                let hndl = insert_rb_with_children(rb, &node, world);

                match hndl {
                    Some(handle) => {
                        let mut casted = node.cast::<RapierPIDCharacter3D>();
                        let node_uid = casted.bind().get_cuid();
                        let raw_handle = handle.into_raw_parts();
                        world.lookup_table.insert(node_uid, raw_handle);
                        casted.bind_mut().set_handle_raw(raw_handle);
                    }
                    None => (),
                };
            }
            "RapierRigidBody3D" => {
                let rb = RigidBodyBuilder::dynamic().position(transform_to_isometry(transform));
                let hndl = insert_rb_with_children(rb, &node, world);

                match hndl {
                    Some(handle) => {
                        let mut casted = node.cast::<RapierRigidBody3D>();
                        let node_uid = casted.bind().get_cuid();
                        let raw_handle = handle.into_raw_parts();
                        world.lookup_table.insert(node_uid, raw_handle);
                        casted.bind_mut().set_handle_raw(raw_handle);
                    }
                    None => (),
                };
            }
            "RapierStaticBody3D" => {
                let rb = RigidBodyBuilder::fixed().position(transform_to_isometry(transform));
                let hndl = insert_rb_with_children(rb, &node, world);

                match hndl {
                    Some(handle) => {
                        let mut casted = node.cast::<RapierStaticBody3D>();
                        let node_uid = casted.bind().get_cuid();
                        let raw_handle = handle.into_raw_parts();
                        world.lookup_table.insert(node_uid, handle.into_raw_parts());
                        casted.bind_mut().set_handle_raw(raw_handle);
                    }
                    None => (),
                };
            }
            "RapierCollisionShape3D" => (), // Ignore colliders - they are inserted at the same time as the parent rigid body
            _ => {
                log::error!("Unknown object type: {}", &class);
            }
        }
    }
}

fn insert_area_children(node: &Node3D, world: &mut World) {
    let colliders = &mut world.physics.colliders;
    let lookup_table = &mut world.lookup_table;

    let children = node
        .find_children_ex("*")
        .type_("RapierCollisionShape3D")
        .recursive(true)
        .owned(false)
        .done();

    if children.len() == 0 {
        log::error!(
            "'{}' must have a child RapierCollisionShape3D",
            node.get_name()
        );
        return;
    }

    for child in children.iter_shared() {
        let mut casted = child.cast::<RapierCollisionShape3D>();
        let child_uid = casted.bind().get_cuid();
        match shape_to_collider(&casted) {
            Some(collider) => {
                let handle = colliders.insert(collider.sensor(true));
                let raw_handle = handle.into_raw_parts();
                lookup_table.insert(child_uid, raw_handle);
                casted.bind_mut().set_handle_raw(raw_handle);
            }
            None => {
                log::error!(
                    "Unknown or missing shape on: '{}'. {}",
                    &casted.get_name(),
                    SUPPORTED_SHAPES
                );
                return;
            }
        }
    }
}

fn insert_rb_with_children(
    rb: impl Into<RigidBody>,
    node: &Node3D,
    world: &mut World,
) -> Option<RigidBodyHandle> {
    let bodies = &mut world.physics.bodies;
    let colliders = &mut world.physics.colliders;
    let lookup_table = &mut world.lookup_table;

    let children = node
        .find_children_ex("*")
        .type_("RapierCollisionShape3D")
        .recursive(true)
        .owned(false)
        .done();

    if children.len() == 0 {
        log::error!(
            "'{}' must have a child RapierCollisionShape3D",
            node.get_name()
        );
        return None;
    }

    let parent_handle = bodies.insert(rb);

    for child in children.iter_shared() {
        let mut casted = child.cast::<RapierCollisionShape3D>();
        let child_uid = casted.bind().get_cuid();
        match shape_to_collider(&casted) {
            Some(collider) => {
                let handle = colliders.insert_with_parent(collider, parent_handle, bodies);
                let raw_handle = handle.into_raw_parts();
                lookup_table.insert(child_uid, raw_handle);
                casted.bind_mut().set_handle_raw(raw_handle);
            }
            None => {
                log::error!(
                    "Unknown or missing shape on: '{}'. {}",
                    &casted.get_name(),
                    SUPPORTED_SHAPES
                );
                return None;
            }
        }
    }

    Some(parent_handle)
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

                let num_verts = &vertices.len();

                let trimesh =
                    ColliderBuilder::trimesh_with_flags(vertices, indices, TriMeshFlags::empty());
                match trimesh {
                    Ok(trimesh) => {
                        let too_many_verts = num_verts > &crate::config::DEBUG_MAX_VERTEX_COUNT;
                        if too_many_verts {
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
        None => None,
    }
}

pub fn remove_nodes_from_world(nodes: Array<Gd<Node3D>>, world: &mut World) {
    for node in nodes.iter_shared() {
        let class = node.get_class().to_string();
        match class.as_str() {
            "RapierArea3D" => {
                let casted = node.cast::<RapierCollisionShape3D>();
                let node_uid = casted.bind().get_cuid();
                if let Some(handle) = world.lookup_table.remove_by_uid(&node_uid) {
                    world.physics.colliders.remove(
                        ColliderHandle::from_raw_parts(handle.0, handle.1),
                        &mut world.physics.islands,
                        &mut world.physics.bodies,
                        false,
                    );
                }
            }
            "RapierKinematicCharacter3D" => {
                let casted = node.cast::<RapierKinematicCharacter3D>();
                let node_uid = casted.bind().get_cuid();
                if let Some(handle) = world.lookup_table.remove_by_uid(&node_uid) {
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
            "RapierRigidBody3D" => {
                let casted = node.cast::<RapierRigidBody3D>();
                let node_uid = casted.bind().get_cuid();
                if let Some(handle) = world.lookup_table.remove_by_uid(&node_uid) {
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
            "RapierStaticBody3D" => {
                let casted = node.cast::<RapierStaticBody3D>();
                let node_uid = casted.bind().get_cuid();
                if let Some(handle) = world.lookup_table.remove_by_uid(&node_uid) {
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
            "RapierCollisionShape3D" => {
                let casted = node.cast::<RapierCollisionShape3D>();
                let node_uid = casted.bind().get_cuid();
                if let Some(handle) = world.lookup_table.remove_by_uid(&node_uid) {
                    world.physics.colliders.remove(
                        ColliderHandle::from_raw_parts(handle.0, handle.1),
                        &mut world.physics.islands,
                        &mut world.physics.bodies,
                        false,
                    );
                }
            }
            _ => {
                log::error!("Unknown object type: {}", &class);
            }
        }
    }
}
