pub mod entities;
pub mod migration;
pub mod service;
pub mod checker;
pub mod scaffolding;

// Re-export komponen utama untuk kemudahan penggunaan di aplikasi akhir
pub use migration::init_permission_tables;
pub use service::PermissionService;
pub use checker::PermissionChecker;

// Re-export entitas agar pengguna bisa melakukan kueri kustom secara langsung
pub use entities::{
    Role, Permission, ModelHasRole, ModelHasPermission, RoleHasPermission,
};
