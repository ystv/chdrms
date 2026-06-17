pub trait Permission {
    const OBJECT: &'static str;
    const ACTION: &'static str;
}

pub struct ObjectPermissions {
    pub object: &'static str,
    pub permissions: &'static [&'static str],
}

pub const ALL_PERMISSIONS: &[ObjectPermissions] = &[
    crate::user::permission::ALL,
    crate::user_group::permission::ALL,
];

macro_rules! define_permission {
    ($s:ident, $object:expr, $action:expr) => {
        pub struct $s;

        impl crate::permission::Permission for $s {
            const OBJECT: &str = $object;
            const ACTION: &str = $action;
        }
    };
}

macro_rules! define_permissions_structs {
    ($object:expr, $action:ident) => {
        define_permission!($action, $object, snake!(stringify!($action)));
    };
    ($object:expr, $action:ident, $($r:ident),+) => {
        define_permissions_structs!($object, $action);
        define_permissions_structs!($object, $($r),+);
    }
}

macro_rules! define_permissions {
    ($object:expr => $($actions:ident),*) => {
        pub mod permission {
            use crate::permission::{define_permission, define_permissions_structs};
            use ::casey::snake;
            define_permissions_structs!($object, $($actions),*);
            pub const ALL: crate::permission::ObjectPermissions = crate::permission::ObjectPermissions {
                object: $object,
                permissions: &[
                    $(snake!(stringify!($actions))),*
                ]
            };
        }
    };
}

pub(crate) use define_permission;
pub(crate) use define_permissions;
pub(crate) use define_permissions_structs;
