use super::HandleKind;
use crate::queue::Actionable;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ObjectBridge {
    pub handle_kind: HandleKind,
    pub object_kind: ObjectKind,
}

impl ObjectBridge {
    pub fn invalid() -> ObjectBridge {
        ObjectBridge {
            handle_kind: HandleKind::Invalid,
            object_kind: ObjectKind::Invalid,
        }
    }
}

// TODO remove object bridge and just do From impl
impl From<ObjectKind> for ObjectBridge {
    fn from(object_kind: ObjectKind) -> Self {
        match object_kind {
            ObjectKind::RigidBody => ObjectBridge {
                handle_kind: HandleKind::RigidBodyHandle,
                object_kind,
            },
            ObjectKind::Collider => ObjectBridge {
                handle_kind: HandleKind::ColliderHandle,
                object_kind,
            },
            ObjectKind::Character => ObjectBridge {
                handle_kind: HandleKind::RigidBodyHandle,
                object_kind,
            },
            ObjectKind::Invalid => ObjectBridge::invalid(),
        }
    }
}

impl From<HandleKind> for ObjectBridge {
    fn from(handle_kind: HandleKind) -> Self {
        match handle_kind {
            HandleKind::RigidBodyHandle => ObjectBridge::from(ObjectKind::RigidBody),
            HandleKind::ColliderHandle => ObjectBridge::from(ObjectKind::Collider),
            HandleKind::Invalid => ObjectBridge::invalid(),
        }
    }
}

impl From<&Actionable> for ObjectBridge {
    fn from(actionable: &Actionable) -> Self {
        match actionable {
            Actionable::RigidBody(_) | Actionable::RigidBodyHandle(_) => {
                ObjectBridge::from(ObjectKind::RigidBody)
            }
            Actionable::Collider(_)
            | Actionable::ColliderWithParent(_, _)
            | Actionable::ColliderIDWithParentID(_, _)
            | Actionable::ColliderHandle(_)
            | Actionable::ColliderShape(_) => ObjectBridge::from(ObjectKind::Collider),
            Actionable::NodePos(kind, _) => ObjectBridge::from(kind.clone()),
            Actionable::Invalid => ObjectBridge::invalid(),
            Actionable::Step => ObjectBridge::invalid(),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum ObjectKind {
    RigidBody,
    Collider,
    Character,
    Invalid,
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
