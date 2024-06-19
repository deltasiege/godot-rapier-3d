use super::HandleKind;
use crate::queue::Actionable;
use std::fmt;

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum ObjectKind {
    RigidBody,
    Collider,
    Character,
    Invalid,
}

impl From<&str> for ObjectKind {
    fn from(kind: &str) -> Self {
        match kind {
            "RigidBody" | "RapierRigidBody3D" => ObjectKind::RigidBody,
            "Collider" | "RapierCollider3D" => ObjectKind::Collider,
            "Character" | "RapierCharacterBody3D" => ObjectKind::Character,
            _ => ObjectKind::Invalid,
        }
    }
}

impl From<&Actionable> for ObjectKind {
    fn from(actionable: &Actionable) -> Self {
        match actionable {
            Actionable::RigidBody(_) | Actionable::RigidBodyHandle(_) => ObjectKind::RigidBody,
            Actionable::Collider(_)
            | Actionable::ColliderWithParent(_, _)
            | Actionable::ColliderIDWithParentID(_, _)
            | Actionable::ColliderHandle(_)
            | Actionable::ColliderShape(_) => ObjectKind::Collider,
            Actionable::NodePos(kind, _) => kind.clone(),
            Actionable::Character(_) | Actionable::MoveCharacter { .. } => ObjectKind::Character,
            Actionable::Invalid => ObjectKind::Invalid,
            Actionable::Step => ObjectKind::Invalid,
        }
    }
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<HandleKind> for ObjectKind {
    fn from(handle_kind: HandleKind) -> Self {
        match handle_kind {
            HandleKind::RigidBodyHandle => ObjectKind::RigidBody,
            HandleKind::ColliderHandle => ObjectKind::Collider,
            HandleKind::Invalid => ObjectKind::Invalid,
        }
    }
}
