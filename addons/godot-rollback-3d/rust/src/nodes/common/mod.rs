mod blueprint;
mod controllable;
mod forceable;
mod identifiable;
mod node_data;
mod rollback_node;

pub use blueprint::{HasBlueprint, NodeBlueprint};
pub use controllable::Controllable;
pub use forceable::Forceable;
pub use identifiable::Identifiable;
pub use node_data::{HasNodeData, NodeData};
pub use rollback_node::RollbackNode;

#[macro_export]
macro_rules! impl_trait_for_all_nodes {
    ($trait_name:ident, $body:tt) => {
        impl $trait_name for crate::nodes::RollbackArea3D $body
        impl $trait_name for crate::nodes::RollbackCollisionShape3D $body
        impl $trait_name for crate::nodes::RollbackKinematicCharacter3D $body
        impl $trait_name for crate::nodes::RollbackPIDCharacter3D $body
        impl $trait_name for crate::nodes::RollbackRigidBody3D $body
        impl $trait_name for crate::nodes::RollbackStaticBody3D $body
    };
}
