use rapier3d::dynamics::RigidBodyHandle;
use rapier3d::geometry::ColliderHandle;
use std::fmt;

use crate::queue::Actionable;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HandleKind {
    RigidBodyHandle,
    ColliderHandle,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handle {
    pub kind: HandleKind,
    pub raw: [u32; 2],
}

impl Handle {
    pub fn new(kind: HandleKind, raw: [u32; 2]) -> Self {
        Handle { kind, raw }
    }

    pub fn invalid() -> Self {
        Handle {
            kind: HandleKind::Invalid,
            raw: [u32::MAX, u32::MAX],
        }
    }
}

impl fmt::Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.raw)
    }
}

impl From<&RigidBodyHandle> for Handle {
    fn from(handle: &RigidBodyHandle) -> Self {
        let parts = handle.into_raw_parts();
        Handle {
            kind: HandleKind::RigidBodyHandle,
            raw: [parts.0, parts.1],
        }
    }
}

impl From<&ColliderHandle> for Handle {
    fn from(handle: &ColliderHandle) -> Self {
        let parts = handle.into_raw_parts();
        Handle {
            kind: HandleKind::ColliderHandle,
            raw: [parts.0, parts.1],
        }
    }
}

impl From<Handle> for RigidBodyHandle {
    fn from(handle: Handle) -> Self {
        RigidBodyHandle::from_raw_parts(handle.raw[0], handle.raw[1])
    }
}

impl From<Handle> for ColliderHandle {
    fn from(handle: Handle) -> Self {
        ColliderHandle::from_raw_parts(handle.raw[0], handle.raw[1])
    }
}

impl From<[u32; 2]> for Handle {
    fn from(raw: [u32; 2]) -> Self {
        Handle {
            kind: HandleKind::Invalid,
            raw,
        }
    }
}

impl From<Actionable> for Handle {
    fn from(actionable: Actionable) -> Self {
        match actionable {
            Actionable::RigidBodyHandle(handle) => Handle::from(&handle),
            Actionable::ColliderHandle(handle) => Handle::from(&handle),
            _ => Handle::invalid(),
        }
    }
}
