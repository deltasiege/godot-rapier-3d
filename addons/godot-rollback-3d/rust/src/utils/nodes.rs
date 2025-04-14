use godot::classes::{
    BoxShape3D, CapsuleShape3D, ConcavePolygonShape3D, CylinderShape3D, SphereShape3D,
};
use godot::prelude::*;
use rapier3d::prelude::*;

use crate::nodes::RollbackCollisionShape3D;
use crate::types::*;
use crate::utils::*;

/// Recursively returns all child under the given root that match the given class.
pub fn get_children_with_class(root: &Gd<Node>, class: RollbackNodeClass) -> Array<Gd<Node>> {
    root.find_children_ex("*")
        .type_(&class.to_string())
        .recursive(true)
        .owned(false)
        .done()
}

/// Returns the node at the given path, or None if it doesn't exist.
pub fn get_node_by_path(other_node: &Gd<Node>, node_path: &String) -> Option<Gd<Node>> {
    let tree = other_node.get_tree()?;
    let parent = tree.get_root()?.get_node_or_null(node_path)?;
    Some(parent)
}

/// Tries to instantiate a resource as a Godot object of type T.
pub fn instantiate_resource_as<T: GodotClass + Inherits<Node>>(
    resource_path: &String,
) -> Option<Gd<T>> {
    let scene = load_scene(resource_path)?;
    let result = scene.try_instantiate_as::<T>();

    if result.is_none() {
        log::error!(
            "Failed to instantiate '{}' as {}",
            resource_path,
            std::any::type_name::<T>(),
        );
    }
    result
}

/// Converts a Transform3D to an Isometry, based on the node's class.
pub fn get_spawn_isometry(node: &Gd<Node3D>, class: &RollbackNodeClass) -> Isometry<Real> {
    match class {
        RollbackNodeClass::RollbackCollisionShape3D => transform_to_isometry(node.get_transform()),
        _ => transform_to_isometry(node.get_global_transform()),
    }
}

/// Returns the RapierBuilder for the given node and class.
pub fn get_rapier_builder(node: &Gd<Node3D>, class: &RollbackNodeClass) -> Option<RapierBuilder> {
    let isometry = get_spawn_isometry(&node, &class);
    match class {
        RollbackNodeClass::RollbackArea3D => None,
        RollbackNodeClass::RollbackKinematicCharacter3D => {
            let rb = RigidBodyBuilder::kinematic_position_based()
                .position(isometry)
                .ccd_enabled(true);
            Some(RapierBuilder::RigidBody(rb.into()))
        }
        RollbackNodeClass::RollbackPIDCharacter3D | RollbackNodeClass::RollbackRigidBody3D => {
            let rb = RigidBodyBuilder::dynamic().position(isometry);
            Some(RapierBuilder::RigidBody(rb.into()))
        }
        RollbackNodeClass::RollbackStaticBody3D => {
            let rb = RigidBodyBuilder::fixed().position(isometry);
            Some(RapierBuilder::RigidBody(rb.into()))
        }
        RollbackNodeClass::RollbackCollisionShape3D => {
            let builder = colshape_to_colbuilder(node.clone().cast::<RollbackCollisionShape3D>())?;
            Some(RapierBuilder::Collider(builder.position(isometry)))
        }
    }
}

/// Converts a RollbackCollisionShape3D to a Rapier ColliderBuilder.
pub fn colshape_to_colbuilder(col_shape: Gd<RollbackCollisionShape3D>) -> Option<ColliderBuilder> {
    let shape = col_shape.bind().get_shape()?;
    let debugging_colliders = is_debugging_colliders(col_shape);
    let builder = match shape.get_class().to_string().as_str() {
        "SphereShape3D" => ColliderBuilder::ball(shape.cast::<SphereShape3D>().get_radius()),
        "BoxShape3D" => {
            let shape = shape.cast::<BoxShape3D>();
            ColliderBuilder::cuboid(
                shape.get_size().x / 2.0,
                shape.get_size().y / 2.0,
                shape.get_size().z / 2.0,
            )
        }
        "CapsuleShape3D" => {
            let shape = shape.cast::<CapsuleShape3D>();
            ColliderBuilder::capsule_y(shape.get_height() / 4.0, shape.get_radius())
        }
        "CylinderShape3D" => {
            let shape = shape.cast::<CylinderShape3D>();
            ColliderBuilder::cylinder(shape.get_height() / 2.0, shape.get_radius())
        }
        "ConcavePolygonShape3D" => {
            shape_to_trimesh(shape.cast::<ConcavePolygonShape3D>(), debugging_colliders)?
        }
        _ => {
            log::error!("Unknown shape class: {}", shape.get_class().to_string());
            return None;
        }
    };
    Some(builder)
}

/// Converts a ConcavePolygonShape3D to a Trimesh ColliderBuilder.
/// Logs a warning if the number of vertices exceeds the maximum allowed
/// for debugging and the user is trying to debug colliders.
pub fn shape_to_trimesh(
    shape: Gd<ConcavePolygonShape3D>,
    debugging_colliders: bool,
) -> Option<ColliderBuilder> {
    let faces = shape.get_faces().to_vec();
    let mut vertices: Vec<Point<f32>> = vec![];
    let mut indices: Vec<[u32; 3]> = vec![];

    for (idx, vert) in faces.iter().enumerate() {
        vertices.push(vector_to_point(&vector_to_rapier(vert.clone())));
        if idx % 3 == 0 {
            indices.push([idx as u32, (idx + 1) as u32, (idx + 2) as u32]);
        }
    }

    if debugging_colliders && vertices.len() > crate::config::DEBUG_MAX_VERTEX_COUNT {
        log::warn!(
            "Debug rendering of collider will be skipped because it has more than {} vertices",
            crate::config::DEBUG_MAX_VERTEX_COUNT
        );
    }

    match ColliderBuilder::trimesh_with_flags(vertices, indices, TriMeshFlags::empty()) {
        Ok(trimesh) => Some(trimesh),
        Err(_) => {
            log::error!("Failed to create trimesh collider from shape: {:?}", shape);
            None
        }
    }
}

/// Returns whether the Debug -> Visible Collision Shapes menu option is enabled.
pub fn is_debugging_colliders(node: Gd<impl Inherits<Node>>) -> bool {
    if let Some(tree) = node.upcast::<Node>().get_tree() {
        tree.is_debugging_collisions_hint()
    } else {
        false
    }
}
