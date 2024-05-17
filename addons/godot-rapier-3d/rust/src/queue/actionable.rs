use std::cmp::Ordering;
use std::fmt;

use crate::objects::{Handle, HandleKind};
use crate::ObjectKind;
use rapier3d::dynamics::{RigidBody, RigidBodyHandle};
use rapier3d::geometry::{Collider, ColliderHandle};
use rapier3d::math::{Isometry, Real};

pub enum Actionable {
    RigidBody(RigidBody),
    RigidBodyHandle(RigidBodyHandle),
    Collider(Collider),
    ColliderWithParent(Collider, RigidBodyHandle),
    ColliderIDWithParentID(String, Option<String>),
    ColliderHandle(ColliderHandle),
    NodePos(ObjectKind, Isometry<Real>),
    Step,
    Invalid,
}

impl Eq for Actionable {}
impl PartialEq for Actionable {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::RigidBody(_), Self::RigidBody(_)) => true,
            (Self::RigidBodyHandle(_), Self::RigidBodyHandle(_)) => true,
            (Self::Collider(_), Self::Collider(_)) => true,
            (Self::ColliderWithParent(_, _), Self::ColliderWithParent(_, _)) => true,
            (Self::ColliderIDWithParentID(_, _), Self::ColliderIDWithParentID(_, _)) => true,
            (Self::ColliderHandle(_), Self::ColliderHandle(_)) => true,
            (Self::NodePos(ObjectKind::RigidBody, _), Self::NodePos(ObjectKind::RigidBody, _)) => {
                true
            }
            (Self::NodePos(ObjectKind::Collider, _), Self::NodePos(ObjectKind::Collider, _)) => {
                true
            }
            (Self::NodePos(ObjectKind::Invalid, _), Self::NodePos(ObjectKind::Invalid, _)) => true,
            _ => false,
        }
    }
}

// Order RigidBodies before Colliders so they are registered first
// (required for parenting colliders to rigid bodies when registering them)
impl PartialOrd for Actionable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (
                Self::RigidBody(_),
                Self::Collider(_)
                | Self::ColliderWithParent(_, _)
                | Self::ColliderIDWithParentID(_, _),
            ) => Some(Ordering::Less),
            (
                Self::Collider(_)
                | Self::ColliderWithParent(_, _)
                | Self::ColliderIDWithParentID(_, _),
                Self::RigidBody(_),
            ) => Some(Ordering::Greater),
            (Self::NodePos(ObjectKind::RigidBody, _), Self::NodePos(ObjectKind::Collider, _)) => {
                Some(Ordering::Less)
            }
            (Self::NodePos(ObjectKind::Collider, _), Self::NodePos(ObjectKind::RigidBody, _)) => {
                Some(Ordering::Greater)
            }
            _ => None,
        }
    }
}
impl Ord for Actionable {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Self::RigidBody(_),
                Self::Collider(_)
                | Self::ColliderWithParent(_, _)
                | Self::ColliderIDWithParentID(_, _),
            ) => Ordering::Less,
            (
                Self::Collider(_)
                | Self::ColliderWithParent(_, _)
                | Self::ColliderIDWithParentID(_, _),
                Self::RigidBody(_),
            ) => Ordering::Greater,
            (Self::NodePos(ObjectKind::RigidBody, _), Self::NodePos(ObjectKind::Collider, _)) => {
                Ordering::Less
            }
            (Self::NodePos(ObjectKind::Collider, _), Self::NodePos(ObjectKind::RigidBody, _)) => {
                Ordering::Greater
            }
            _ => Ordering::Equal,
        }
    }
}

impl fmt::Debug for Actionable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RigidBody(rb) => write!(f, "RigidBody: {:?}", rb),
            Self::RigidBodyHandle(handle) => write!(f, "RigidBodyHandle: {:?}", handle),
            Self::Collider(col) => write!(f, "Collider: {:?}", col.shared_shape()),
            Self::ColliderWithParent(c, h) => {
                write!(f, "ColliderWithParent: {:?} {:?}", c.shared_shape(), h)
            }
            Self::ColliderIDWithParentID(handle, parent) => {
                write!(f, "ColliderIDWithParentID: {:?} {:?}", handle, parent)
            }
            Self::ColliderHandle(handle) => write!(f, "ColliderHandle: {:?}", handle),
            Self::NodePos(kind, pos) => write!(f, "NodePos: {:?} {:?}", kind, pos),
            Self::Step => write!(f, "Step"),
            Self::Invalid => write!(f, "Invalid"),
        }
    }
}

impl fmt::Display for Actionable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RigidBody(_) => write!(f, "Actionable::RigidBody"),
            Self::RigidBodyHandle(_) => write!(f, "Actionable::RigidBodyHandle"),
            Self::Collider(_) => write!(f, "Actionable::Collider"),
            Self::ColliderWithParent(_, _) => write!(f, "Actionable::ColliderWithParent"),
            Self::ColliderIDWithParentID(_, _) => {
                write!(f, "Actionable::ColliderIDWithParentID")
            }
            Self::ColliderHandle(_) => write!(f, "Actionable::ColliderHandle"),
            Self::NodePos(_, _) => write!(f, "Actionable::NodePos"),
            Self::Step => write!(f, "Actionable::Step"),
            Self::Invalid => write!(f, "Actionable::Invalid"),
        }
    }
}

impl From<Handle> for Actionable {
    fn from(handle: Handle) -> Self {
        match handle.kind {
            HandleKind::RigidBodyHandle => {
                Actionable::RigidBodyHandle(RigidBodyHandle::from(handle))
            }
            HandleKind::ColliderHandle => Actionable::ColliderHandle(ColliderHandle::from(handle)),
            HandleKind::Invalid => Actionable::Invalid,
        }
    }
}

impl From<Actionable> for Collider {
    fn from(actionable: Actionable) -> Self {
        match actionable {
            Actionable::Collider(col) => col,
            _ => panic!("Actionable::Collider expected"),
        }
    }
}
