use rustbasic_core::sql::{self, AnyPool};
use rustbasic_core::chrono::Utc;
use crate::entities::{Role, Permission};

pub struct PermissionService;

fn map_row<T: rustbasic_core::serde::de::DeserializeOwned>(row: &sql::any::AnyRow) -> Result<T, sql::Error> {
    let val = rustbasic_core::database::row_to_json_value(row);
    rustbasic_core::serde_json::from_value::<T>(val)
        .map_err(|e| sql::Error::Protocol(format!("Deserialization error: {}", e)))
}

impl PermissionService {
    /// Mencari atau membuat Role baru berdasarkan nama.
    pub async fn create_role(
        db: &AnyPool,
        name: &str,
        guard_name: Option<&str>,
    ) -> Result<Role, sql::Error> {
        let guard = guard_name.unwrap_or("web");
        
        // Cek apakah role sudah ada
        if let Some(row) = sql::query::<sql::Any>("SELECT * FROM roles WHERE name = ?")
            .bind(name)
            .fetch_optional(db)
            .await?
        {
            return map_row(&row);
        }

        // Buat role baru
        let now = Utc::now().naive_utc().to_string();
        sql::query::<sql::Any>("INSERT INTO roles (name, guard_name, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind(name)
            .bind(guard)
            .bind(&now)
            .bind(&now)
            .execute(db)
            .await?;
        
        // Ambil kembali data lengkap yang baru diinsert
        let row = sql::query::<sql::Any>("SELECT * FROM roles WHERE name = ?")
            .bind(name)
            .fetch_one(db)
            .await?;
        map_row(&row)
    }

    /// Mencari atau membuat Permission baru berdasarkan nama.
    pub async fn create_permission(
        db: &AnyPool,
        name: &str,
        guard_name: Option<&str>,
    ) -> Result<Permission, sql::Error> {
        let guard = guard_name.unwrap_or("web");
        
        if let Some(row) = sql::query::<sql::Any>("SELECT * FROM permissions WHERE name = ?")
            .bind(name)
            .fetch_optional(db)
            .await?
        {
            return map_row(&row);
        }

        let now = Utc::now().naive_utc().to_string();
        sql::query::<sql::Any>("INSERT INTO permissions (name, guard_name, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind(name)
            .bind(guard)
            .bind(&now)
            .bind(&now)
            .execute(db)
            .await?;
        
        let row = sql::query::<sql::Any>("SELECT * FROM permissions WHERE name = ?")
            .bind(name)
            .fetch_one(db)
            .await?;
        map_row(&row)
    }

    /// Menetapkan (assign) Role ke suatu entitas model (misal: user).
    pub async fn assign_role(
        db: &AnyPool,
        model_type: &str,
        model_id: i32,
        role_name: &str,
    ) -> Result<(), sql::Error> {
        let role_row = sql::query::<sql::Any>("SELECT * FROM roles WHERE name = ?")
            .bind(role_name)
            .fetch_optional(db)
            .await?
            .ok_or(sql::Error::RowNotFound)?;
        let role = map_row::<Role>(&role_row)?;

        // Cek apakah sudah di-assign sebelumnya
        let existing = sql::query::<sql::Any>("SELECT 1 FROM model_has_roles WHERE role_id = ? AND model_type = ? AND model_id = ?")
            .bind(role.id)
            .bind(model_type)
            .bind(model_id)
            .fetch_optional(db)
            .await?;

        if existing.is_none() {
            sql::query::<sql::Any>("INSERT INTO model_has_roles (role_id, model_type, model_id) VALUES (?, ?, ?)")
                .bind(role.id)
                .bind(model_type)
                .bind(model_id)
                .execute(db)
                .await?;
        }

        Ok(())
    }

    /// Menghapus (remove) Role dari suatu entitas model.
    pub async fn remove_role(
        db: &AnyPool,
        model_type: &str,
        model_id: i32,
        role_name: &str,
    ) -> Result<(), sql::Error> {
        let role_row = match sql::query::<sql::Any>("SELECT * FROM roles WHERE name = ?")
            .bind(role_name)
            .fetch_optional(db)
            .await?
        {
            Some(r) => r,
            None => return Ok(()), // Role tidak ada, anggap sukses dihapus
        };
        let role = map_row::<Role>(&role_row)?;

        sql::query::<sql::Any>("DELETE FROM model_has_roles WHERE role_id = ? AND model_type = ? AND model_id = ?")
            .bind(role.id)
            .bind(model_type)
            .bind(model_id)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Memeriksa apakah model memiliki Role tertentu.
    pub async fn has_role(
        db: &AnyPool,
        model_type: &str,
        model_id: i32,
        role_name: &str,
    ) -> Result<bool, sql::Error> {
        let role_row = match sql::query::<sql::Any>("SELECT * FROM roles WHERE name = ?")
            .bind(role_name)
            .fetch_optional(db)
            .await?
        {
            Some(r) => r,
            None => return Ok(false),
        };
        let role = map_row::<Role>(&role_row)?;

        let existing = sql::query::<sql::Any>("SELECT 1 FROM model_has_roles WHERE role_id = ? AND model_type = ? AND model_id = ?")
            .bind(role.id)
            .bind(model_type)
            .bind(model_id)
            .fetch_optional(db)
            .await?;

        Ok(existing.is_some())
    }

    /// Menetapkan Permission langsung ke suatu entitas model.
    pub async fn give_permission_to(
        db: &AnyPool,
        model_type: &str,
        model_id: i32,
        permission_name: &str,
    ) -> Result<(), sql::Error> {
        let perm_row = sql::query::<sql::Any>("SELECT * FROM permissions WHERE name = ?")
            .bind(permission_name)
            .fetch_optional(db)
            .await?
            .ok_or(sql::Error::RowNotFound)?;
        let perm = map_row::<Permission>(&perm_row)?;

        let existing = sql::query::<sql::Any>("SELECT 1 FROM model_has_permissions WHERE permission_id = ? AND model_type = ? AND model_id = ?")
            .bind(perm.id)
            .bind(model_type)
            .bind(model_id)
            .fetch_optional(db)
            .await?;

        if existing.is_none() {
            sql::query::<sql::Any>("INSERT INTO model_has_permissions (permission_id, model_type, model_id) VALUES (?, ?, ?)")
                .bind(perm.id)
                .bind(model_type)
                .bind(model_id)
                .execute(db)
                .await?;
        }

        Ok(())
    }

    /// Mencabut Permission langsung dari suatu entitas model.
    pub async fn revoke_permission_to(
        db: &AnyPool,
        model_type: &str,
        model_id: i32,
        permission_name: &str,
    ) -> Result<(), sql::Error> {
        let perm_row = match sql::query::<sql::Any>("SELECT * FROM permissions WHERE name = ?")
            .bind(permission_name)
            .fetch_optional(db)
            .await?
        {
            Some(p) => p,
            None => return Ok(()),
        };
        let perm = map_row::<Permission>(&perm_row)?;

        sql::query::<sql::Any>("DELETE FROM model_has_permissions WHERE permission_id = ? AND model_type = ? AND model_id = ?")
            .bind(perm.id)
            .bind(model_type)
            .bind(model_id)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Menetapkan Permission ke sebuah Role.
    pub async fn give_permission_to_role(
        db: &AnyPool,
        role_name: &str,
        permission_name: &str,
    ) -> Result<(), sql::Error> {
        let role_row = sql::query::<sql::Any>("SELECT * FROM roles WHERE name = ?")
            .bind(role_name)
            .fetch_optional(db)
            .await?
            .ok_or(sql::Error::RowNotFound)?;
        let role = map_row::<Role>(&role_row)?;

        let perm_row = sql::query::<sql::Any>("SELECT * FROM permissions WHERE name = ?")
            .bind(permission_name)
            .fetch_optional(db)
            .await?
            .ok_or(sql::Error::RowNotFound)?;
        let perm = map_row::<Permission>(&perm_row)?;

        let existing = sql::query::<sql::Any>("SELECT 1 FROM role_has_permissions WHERE role_id = ? AND permission_id = ?")
            .bind(role.id)
            .bind(perm.id)
            .fetch_optional(db)
            .await?;

        if existing.is_none() {
            sql::query::<sql::Any>("INSERT INTO role_has_permissions (role_id, permission_id) VALUES (?, ?)")
                .bind(role.id)
                .bind(perm.id)
                .execute(db)
                .await?;
        }

        Ok(())
    }

    /// Mencabut Permission dari sebuah Role.
    pub async fn revoke_permission_from_role(
        db: &AnyPool,
        role_name: &str,
        permission_name: &str,
    ) -> Result<(), sql::Error> {
        let role_row = match sql::query::<sql::Any>("SELECT * FROM roles WHERE name = ?")
            .bind(role_name)
            .fetch_optional(db)
            .await?
        {
            Some(r) => r,
            None => return Ok(()),
        };
        let role = map_row::<Role>(&role_row)?;

        let perm_row = match sql::query::<sql::Any>("SELECT * FROM permissions WHERE name = ?")
            .bind(permission_name)
            .fetch_optional(db)
            .await?
        {
            Some(p) => p,
            None => return Ok(()),
        };
        let perm = map_row::<Permission>(&perm_row)?;

        sql::query::<sql::Any>("DELETE FROM role_has_permissions WHERE role_id = ? AND permission_id = ?")
            .bind(role.id)
            .bind(perm.id)
            .execute(db)
            .await?;

        Ok(())
    }

    /// Memeriksa apakah model memiliki Permission tertentu (baik langsung maupun via Role).
    pub async fn has_permission_to(
        db: &AnyPool,
        model_type: &str,
        model_id: i32,
        permission_name: &str,
    ) -> Result<bool, sql::Error> {
        let perm_row = match sql::query::<sql::Any>("SELECT * FROM permissions WHERE name = ?")
            .bind(permission_name)
            .fetch_optional(db)
            .await?
        {
            Some(p) => p,
            None => return Ok(false),
        };
        let perm = map_row::<Permission>(&perm_row)?;

        // 1. Cek izin langsung di model_has_permissions
        let direct = sql::query::<sql::Any>("SELECT 1 FROM model_has_permissions WHERE permission_id = ? AND model_type = ? AND model_id = ?")
            .bind(perm.id)
            .bind(model_type)
            .bind(model_id)
            .fetch_optional(db)
            .await?;

        if direct.is_some() {
            return Ok(true);
        }

        // 2. Cek izin melalui peran (roles) yang dimiliki model
        let has_perm_via_role = sql::query::<sql::Any>(
            "SELECT 1 FROM role_has_permissions rhp \
             JOIN model_has_roles mhr ON rhp.role_id = mhr.role_id \
             WHERE mhr.model_type = ? AND mhr.model_id = ? AND rhp.permission_id = ?"
        )
        .bind(model_type)
        .bind(model_id)
        .bind(perm.id)
        .fetch_optional(db)
        .await?;

        Ok(has_perm_via_role.is_some())
    }

    /// Mengambil daftar semua nama Role yang dimiliki oleh sebuah model.
    pub async fn get_all_roles_for(
        db: &AnyPool,
        model_type: &str,
        model_id: i32,
    ) -> Result<Vec<String>, sql::Error> {
        let rows = sql::query::<sql::Any>("SELECT r.name FROM roles r JOIN model_has_roles mhr ON r.id = mhr.role_id WHERE mhr.model_type = ? AND mhr.model_id = ?")
            .bind(model_type)
            .bind(model_id)
            .fetch_all(db)
            .await?;

        Ok(rows.into_iter().map(|row| row.get::<String, &str>("name")).collect())
    }

    /// Mengambil daftar semua nama Permission yang dimiliki oleh sebuah model (langsung maupun via Role).
    /// Jika model memiliki role "admin", otomatis mengembalikan semua permission yang ada di sistem.
    pub async fn get_all_permissions_for(
        db: &AnyPool,
        model_type: &str,
        model_id: i32,
    ) -> Result<Vec<String>, sql::Error> {
        // Cek apakah user adalah admin
        let roles = Self::get_all_roles_for(db, model_type, model_id).await?;
        if roles.iter().any(|r| r == "admin") {
            let rows = sql::query::<sql::Any>("SELECT name FROM permissions")
                .fetch_all(db)
                .await?;
            return Ok(rows.into_iter().map(|row| row.get::<String, &str>("name")).collect());
        }

        // Ambil all permissions (direct + via roles)
        let rows = sql::query::<sql::Any>(
            "SELECT p.name FROM permissions p \
             JOIN model_has_permissions mhp ON p.id = mhp.permission_id \
             WHERE mhp.model_type = ? AND mhp.model_id = ? \
             UNION \
             SELECT p.name FROM permissions p \
             JOIN role_has_permissions rhp ON p.id = rhp.permission_id \
             JOIN model_has_roles mhr ON rhp.role_id = mhr.role_id \
             WHERE mhr.model_type = ? AND mhr.model_id = ?"
        )
        .bind(model_type)
        .bind(model_id)
        .bind(model_type)
        .bind(model_id)
        .fetch_all(db)
        .await?;

        Ok(rows.into_iter().map(|row| row.get::<String, &str>("name")).collect())
    }
}
