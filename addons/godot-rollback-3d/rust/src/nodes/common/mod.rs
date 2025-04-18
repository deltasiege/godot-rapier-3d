mod blueprint;
mod controllable;
mod forceable;
mod node_data;
mod rollback_node;

pub use blueprint::*;
pub use controllable::*;
pub use forceable::Forceable;
pub use node_data::{HasNodeData, NodeData};
pub use rollback_node::RollbackNode;

macro_rules! impl_trait_for_nodes {
    ($trait_name:ident, $body:tt, $($node:ty),+) => {
        $(impl $trait_name for $node $body)+
    };
}
pub(crate) use impl_trait_for_nodes;

macro_rules! impl_trait_for_all_nodes {
    ($trait_name:ident, $body:tt) => {
        impl $trait_name for $crate::nodes::RollbackArea3D $body
        impl $trait_name for $crate::nodes::RollbackCollisionShape3D $body
        impl $trait_name for $crate::nodes::RollbackKinematicCharacter3D $body
        impl $trait_name for $crate::nodes::RollbackPIDCharacter3D $body
        impl $trait_name for $crate::nodes::RollbackRigidBody3D $body
        impl $trait_name for $crate::nodes::RollbackStaticBody3D $body
    };
}
pub(crate) use impl_trait_for_all_nodes;
