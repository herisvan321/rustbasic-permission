use rustbasic_core::sql::{self, AnyPool};
use rustbasic_core::session::Session;
use rustbasic_core::responses::ResponseHelper;
use rustbasic_core::router::Response;
use crate::service::PermissionService;

/// Helper `PermissionChecker` dirancang digunakan langsung di dalam controller RustBasic
/// guna memverifikasi hak akses pengguna yang sedang aktif di session.
pub struct PermissionChecker<'a> {
    pub db: &'a AnyPool,
    pub session: &'a Session,
    pub model_type: &'a str,
}

impl<'a> PermissionChecker<'a> {
    /// Membuat instance `PermissionChecker` dengan target model_type default `"users"`.
    pub fn new(db: &'a AnyPool, session: &'a Session) -> Self {
        Self {
            db,
            session,
            model_type: "users",
        }
    }

    /// Membuat instance `PermissionChecker` dengan model_type kustom.
    pub fn with_model_type(
        db: &'a AnyPool,
        session: &'a Session,
        model_type: &'a str,
    ) -> Self {
        Self {
            db,
            session,
            model_type,
        }
    }

    /// Mengambil `user_id` dari session yang sedang terotentikasi.
    pub fn user_id(&self) -> Option<i32> {
        self.session.get::<i32>("user_id")
    }

    /// Memeriksa apakah user yang sedang login memiliki Role tertentu.
    pub async fn has_role(&self, role_name: &str) -> Result<bool, sql::Error> {
        let uid = match self.user_id() {
            Some(id) => id,
            None => return Ok(false),
        };
        PermissionService::has_role(self.db, self.model_type, uid, role_name).await
    }

    /// Memeriksa apakah user yang sedang login memiliki Permission tertentu.
    pub async fn has_permission(&self, permission_name: &str) -> Result<bool, sql::Error> {
        let uid = match self.user_id() {
            Some(id) => id,
            None => return Ok(false),
        };
        PermissionService::has_permission_to(self.db, self.model_type, uid, permission_name).await
    }

    /// Helper praktis: Memastikan user aktif memiliki Role yang diminta.
    /// Jika gagal, akan langsung mengembalikan HTTP Response redirect dengan Flash Error.
    pub async fn require_role(&self, role_name: &str, redirect_to: &str) -> Result<(), Response> {
        match self.has_role(role_name).await {
            Ok(true) => Ok(()),
            _ => Err(ResponseHelper::redirect_with_error(
                redirect_to,
                &format!("Akses ditolak: Diperlukan role '{}'", role_name),
                self.session.clone(),
            )),
        }
    }

    /// Helper praktis: Memastikan user aktif memiliki Permission yang diminta.
    /// Jika gagal, akan langsung mengembalikan HTTP Response redirect dengan Flash Error.
    pub async fn require_permission(&self, permission_name: &str, redirect_to: &str) -> Result<(), Response> {
        match self.has_permission(permission_name).await {
            Ok(true) => Ok(()),
            _ => Err(ResponseHelper::redirect_with_error(
                redirect_to,
                &format!("Akses ditolak: Diperlukan izin '{}'", permission_name),
                self.session.clone(),
            )),
        }
    }

    /// Mengambil daftar semua nama Permission yang dimiliki oleh user aktif di session.
    pub async fn user_permissions(&self) -> Vec<String> {
        let uid = match self.user_id() {
            Some(id) => id,
            None => return vec![],
        };
        PermissionService::get_all_permissions_for(self.db, self.model_type, uid).await.unwrap_or_default()
    }

    /// Mengambil daftar semua nama Role yang dimiliki oleh user aktif di session.
    pub async fn user_roles(&self) -> Vec<String> {
        let uid = match self.user_id() {
            Some(id) => id,
            None => return vec![],
        };
        PermissionService::get_all_roles_for(self.db, self.model_type, uid).await.unwrap_or_default()
    }
}
