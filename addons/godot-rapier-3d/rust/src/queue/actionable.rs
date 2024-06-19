use std::cmp::Ordering;
use std::fmt;

use crate::objects::{Handle, HandleKind};
use crate::ObjectKind;
use rapier3d::control::KinematicCharacterController;
use rapier3d::dynamics::{RigidBody, RigidBodyHandle};
use rapier3d::geometry::{Collider, ColliderHandle, SharedShape};
use rapier3d::math::{Isometry, Real, Vector};

pub enum Actionable {
    RigidBody(RigidBody),
    RigidBodyHandle(RigidBodyHandle),
    Collider(Collider),
    ColliderWithParent(Collider, RigidBodyHandle),
    ColliderIDWithParentID(String, Option<String>),
    ColliderShape(SharedShape),
    ColliderHandle(ColliderHandle),
    NodePos(ObjectKind, Isometry<Real>),
    Character(RigidBody),
    MoveCharacter {
        controller: KinematicCharacterController,
        cuid2: String,
        amount: Vector<Real>,
        delta_time: Real,
    },
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

/*
    1. Order RigidBodies and Characters before Colliders so they are registered first
    (required for parenting colliders to rigid bodies when registering them)

    2. Put step actions last in sim queue so that characters move before other bodies are simulated
*/
impl PartialOrd for Actionable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (
                Self::RigidBody(_) | Self::Character(_),
                Self::Collider(_)
                | Self::ColliderWithParent(_, _)
                | Self::ColliderIDWithParentID(_, _),
            ) => Some(Ordering::Less),
            (
                Self::Collider(_)
                | Self::ColliderWithParent(_, _)
                | Self::ColliderIDWithParentID(_, _),
                Self::RigidBody(_) | Self::Character(_),
            ) => Some(Ordering::Greater),
            (
                Self::NodePos(ObjectKind::RigidBody | ObjectKind::Character, _),
                Self::NodePos(ObjectKind::Collider, _),
            ) => Some(Ordering::Less),
            (
                Self::NodePos(ObjectKind::Collider, _),
                Self::NodePos(ObjectKind::RigidBody | ObjectKind::Character, _),
            ) => Some(Ordering::Greater),
            (Self::Step, _) => Some(Ordering::Greater),
            (_, Self::Step) => Some(Ordering::Less),
            _ => None,
        }
    }
}
impl Ord for Actionable {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Self::RigidBody(_) | Self::Character(_),
                Self::Collider(_)
                | Self::ColliderWithParent(_, _)
                | Self::ColliderIDWithParentID(_, _),
            ) => Ordering::Less,
            (
                Self::Collider(_)
                | Self::ColliderWithParent(_, _)
                | Self::ColliderIDWithParentID(_, _),
                Self::RigidBody(_) | Self::Character(_),
            ) => Ordering::Greater,
            (
                Self::NodePos(ObjectKind::RigidBody | ObjectKind::Character, _),
                Self::NodePos(ObjectKind::Collider, _),
            ) => Ordering::Less,
            (
                Self::NodePos(ObjectKind::Collider, _),
                Self::NodePos(ObjectKind::RigidBody | ObjectKind::Character, _),
            ) => Ordering::Greater,
            (Self::Step, _) => Ordering::Greater,
            (_, Self::Step) => Ordering::Less,
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
            Self::ColliderShape(shape) => write!(f, "ColliderShape: {:?}", shape),
            Self::ColliderHandle(handle) => write!(f, "ColliderHandle: {:?}", handle),
            Self::NodePos(kind, pos) => write!(f, "NodePos: {:?} {:?}", kind, pos),
            Self::Character(rb) => write!(f, "Character: {:?}", rb),
            Self::MoveCharacter {
                cuid2,
                amount,
                delta_time,
                ..
            } => {
                write!(
                    f,
                    "MoveCharacter: '{:?}' {:?} {:?}",
                    cuid2, amount, delta_time
                )
            }
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
            Self::ColliderShape(_) => write!(f, "Actionable::ColliderShape"),
            Self::ColliderHandle(_) => write!(f, "Actionable::ColliderHandle"),
            Self::NodePos(_, _) => write!(f, "Actionable::NodePos"),
            Self::Character(_) => write!(f, "Actionable::Character"),
            Self::MoveCharacter { .. } => write!(f, "Actionable::MoveCharacter"),
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
