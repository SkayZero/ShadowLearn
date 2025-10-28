pub mod checker;

pub use checker::{
    check_permissions, request_accessibility_permission, request_screen_recording_permission,
    PermissionChecker, PermissionStatus,
};
