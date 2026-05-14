pub mod entities;
pub mod migration;
pub mod service;
pub mod checker;

// Re-export komponen utama untuk kemudahan penggunaan di aplikasi akhir
pub use migration::init_permission_tables;
pub use service::PermissionService;
pub use checker::PermissionChecker;

// Re-export entitas SeaORM agar pengguna bisa melakukan kueri kustom secara langsung
pub use entities::{
    roles, permissions, model_has_roles, model_has_permissions, role_has_permissions,
};
