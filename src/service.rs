use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, QueryFilter, ColumnTrait, Set,
};
use rustbasic_core::chrono::Utc;
use crate::entities::{
    roles, permissions, model_has_roles, model_has_permissions, role_has_permissions,
};

pub struct PermissionService;

impl PermissionService {
    /// Mencari atau membuat Role baru berdasarkan nama.
    pub async fn create_role(
        db: &DatabaseConnection,
        name: &str,
        guard_name: Option<&str>,
    ) -> Result<roles::Model, DbErr> {
        let guard = guard_name.unwrap_or("web");
        
        // Cek apakah role sudah ada
        if let Some(existing) = roles::Entity::find()
            .filter(roles::Column::Name.eq(name))
            .one(db)
            .await?
        {
            return Ok(existing);
        }

        // Buat role baru
        let now = Utc::now().naive_utc();
        let new_role = roles::ActiveModel {
            name: Set(name.to_string()),
            guard_name: Set(guard.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let res = roles::Entity::insert(new_role).exec(db).await?;
        
        // Ambil kembali data lengkap yang baru diinsert
        roles::Entity::find_by_id(res.last_insert_id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Gagal mengambil role setelah insert".into()))
    }

    /// Mencari atau membuat Permission baru berdasarkan nama.
    pub async fn create_permission(
        db: &DatabaseConnection,
        name: &str,
        guard_name: Option<&str>,
    ) -> Result<permissions::Model, DbErr> {
        let guard = guard_name.unwrap_or("web");
        
        if let Some(existing) = permissions::Entity::find()
            .filter(permissions::Column::Name.eq(name))
            .one(db)
            .await?
        {
            return Ok(existing);
        }

        let now = Utc::now().naive_utc();
        let new_perm = permissions::ActiveModel {
            name: Set(name.to_string()),
            guard_name: Set(guard.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let res = permissions::Entity::insert(new_perm).exec(db).await?;
        
        permissions::Entity::find_by_id(res.last_insert_id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Gagal mengambil permission setelah insert".into()))
    }

    /// Menetapkan (assign) Role ke suatu entitas model (misal: user).
    pub async fn assign_role(
        db: &DatabaseConnection,
        model_type: &str,
        model_id: i32,
        role_name: &str,
    ) -> Result<(), DbErr> {
        let role = roles::Entity::find()
            .filter(roles::Column::Name.eq(role_name))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom(format!("Role '{}' tidak ditemukan", role_name)))?;

        // Cek apakah sudah di-assign sebelumnya
        let existing = model_has_roles::Entity::find()
            .filter(model_has_roles::Column::RoleId.eq(role.id))
            .filter(model_has_roles::Column::ModelType.eq(model_type))
            .filter(model_has_roles::Column::ModelId.eq(model_id))
            .one(db)
            .await?;

        if existing.is_none() {
            let pivot = model_has_roles::ActiveModel {
                role_id: Set(role.id),
                model_type: Set(model_type.to_string()),
                model_id: Set(model_id),
                ..Default::default()
            };
            model_has_roles::Entity::insert(pivot).exec(db).await?;
        }

        Ok(())
    }

    /// Menghapus (remove) Role dari suatu entitas model.
    pub async fn remove_role(
        db: &DatabaseConnection,
        model_type: &str,
        model_id: i32,
        role_name: &str,
    ) -> Result<(), DbErr> {
        let role = match roles::Entity::find()
            .filter(roles::Column::Name.eq(role_name))
            .one(db)
            .await?
        {
            Some(r) => r,
            None => return Ok(()), // Role tidak ada, anggap sukses dihapus
        };

        // Hapus entri dari model_has_roles
        model_has_roles::Entity::delete_many()
            .filter(model_has_roles::Column::RoleId.eq(role.id))
            .filter(model_has_roles::Column::ModelType.eq(model_type))
            .filter(model_has_roles::Column::ModelId.eq(model_id))
            .exec(db)
            .await?;

        Ok(())
    }

    /// Memeriksa apakah model memiliki Role tertentu.
    pub async fn has_role(
        db: &DatabaseConnection,
        model_type: &str,
        model_id: i32,
        role_name: &str,
    ) -> Result<bool, DbErr> {
        let role = match roles::Entity::find()
            .filter(roles::Column::Name.eq(role_name))
            .one(db)
            .await?
        {
            Some(r) => r,
            None => return Ok(false),
        };

        let existing = model_has_roles::Entity::find()
            .filter(model_has_roles::Column::RoleId.eq(role.id))
            .filter(model_has_roles::Column::ModelType.eq(model_type))
            .filter(model_has_roles::Column::ModelId.eq(model_id))
            .one(db)
            .await?;

        Ok(existing.is_some())
    }

    /// Menetapkan Permission langsung ke suatu entitas model.
    pub async fn give_permission_to(
        db: &DatabaseConnection,
        model_type: &str,
        model_id: i32,
        permission_name: &str,
    ) -> Result<(), DbErr> {
        let perm = permissions::Entity::find()
            .filter(permissions::Column::Name.eq(permission_name))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom(format!("Permission '{}' tidak ditemukan", permission_name)))?;

        let existing = model_has_permissions::Entity::find()
            .filter(model_has_permissions::Column::PermissionId.eq(perm.id))
            .filter(model_has_permissions::Column::ModelType.eq(model_type))
            .filter(model_has_permissions::Column::ModelId.eq(model_id))
            .one(db)
            .await?;

        if existing.is_none() {
            let pivot = model_has_permissions::ActiveModel {
                permission_id: Set(perm.id),
                model_type: Set(model_type.to_string()),
                model_id: Set(model_id),
                ..Default::default()
            };
            model_has_permissions::Entity::insert(pivot).exec(db).await?;
        }

        Ok(())
    }

    /// Mencabut Permission langsung dari suatu entitas model.
    pub async fn revoke_permission_to(
        db: &DatabaseConnection,
        model_type: &str,
        model_id: i32,
        permission_name: &str,
    ) -> Result<(), DbErr> {
        let perm = match permissions::Entity::find()
            .filter(permissions::Column::Name.eq(permission_name))
            .one(db)
            .await?
        {
            Some(p) => p,
            None => return Ok(()),
        };

        model_has_permissions::Entity::delete_many()
            .filter(model_has_permissions::Column::PermissionId.eq(perm.id))
            .filter(model_has_permissions::Column::ModelType.eq(model_type))
            .filter(model_has_permissions::Column::ModelId.eq(model_id))
            .exec(db)
            .await?;

        Ok(())
    }

    /// Menetapkan Permission ke sebuah Role.
    pub async fn give_permission_to_role(
        db: &DatabaseConnection,
        role_name: &str,
        permission_name: &str,
    ) -> Result<(), DbErr> {
        let role = roles::Entity::find()
            .filter(roles::Column::Name.eq(role_name))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom(format!("Role '{}' tidak ditemukan", role_name)))?;

        let perm = permissions::Entity::find()
            .filter(permissions::Column::Name.eq(permission_name))
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom(format!("Permission '{}' tidak ditemukan", permission_name)))?;

        let existing = role_has_permissions::Entity::find()
            .filter(role_has_permissions::Column::RoleId.eq(role.id))
            .filter(role_has_permissions::Column::PermissionId.eq(perm.id))
            .one(db)
            .await?;

        if existing.is_none() {
            let pivot = role_has_permissions::ActiveModel {
                permission_id: Set(perm.id),
                role_id: Set(role.id),
                ..Default::default()
            };
            role_has_permissions::Entity::insert(pivot).exec(db).await?;
        }

        Ok(())
    }

    /// Mencabut Permission dari sebuah Role.
    pub async fn revoke_permission_from_role(
        db: &DatabaseConnection,
        role_name: &str,
        permission_name: &str,
    ) -> Result<(), DbErr> {
        let role = match roles::Entity::find()
            .filter(roles::Column::Name.eq(role_name))
            .one(db)
            .await?
        {
            Some(r) => r,
            None => return Ok(()),
        };

        let perm = match permissions::Entity::find()
            .filter(permissions::Column::Name.eq(permission_name))
            .one(db)
            .await?
        {
            Some(p) => p,
            None => return Ok(()),
        };

        role_has_permissions::Entity::delete_many()
            .filter(role_has_permissions::Column::RoleId.eq(role.id))
            .filter(role_has_permissions::Column::PermissionId.eq(perm.id))
            .exec(db)
            .await?;

        Ok(())
    }

    /// Memeriksa apakah model memiliki Permission tertentu (baik langsung maupun via Role).
    pub async fn has_permission_to(
        db: &DatabaseConnection,
        model_type: &str,
        model_id: i32,
        permission_name: &str,
    ) -> Result<bool, DbErr> {
        let perm = match permissions::Entity::find()
            .filter(permissions::Column::Name.eq(permission_name))
            .one(db)
            .await?
        {
            Some(p) => p,
            None => return Ok(false),
        };

        // 1. Cek izin langsung di model_has_permissions
        let direct = model_has_permissions::Entity::find()
            .filter(model_has_permissions::Column::PermissionId.eq(perm.id))
            .filter(model_has_permissions::Column::ModelType.eq(model_type))
            .filter(model_has_permissions::Column::ModelId.eq(model_id))
            .one(db)
            .await?;

        if direct.is_some() {
            return Ok(true);
        }

        // 2. Cek izin melalui peran (roles) yang dimiliki model
        let assigned_roles = model_has_roles::Entity::find()
            .filter(model_has_roles::Column::ModelType.eq(model_type))
            .filter(model_has_roles::Column::ModelId.eq(model_id))
            .all(db)
            .await?;

        for role_pivot in assigned_roles {
            let has_perm_via_role = role_has_permissions::Entity::find()
                .filter(role_has_permissions::Column::RoleId.eq(role_pivot.role_id))
                .filter(role_has_permissions::Column::PermissionId.eq(perm.id))
                .one(db)
                .await?;

            if has_perm_via_role.is_some() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Mengambil daftar semua nama Role yang dimiliki oleh sebuah model.
    pub async fn get_all_roles_for(
        db: &DatabaseConnection,
        model_type: &str,
        model_id: i32,
    ) -> Result<Vec<String>, DbErr> {
        let pivots = model_has_roles::Entity::find()
            .filter(model_has_roles::Column::ModelType.eq(model_type))
            .filter(model_has_roles::Column::ModelId.eq(model_id))
            .all(db)
            .await?;

        let mut role_names = Vec::new();
        for p in pivots {
            if let Some(r) = roles::Entity::find_by_id(p.role_id).one(db).await? {
                role_names.push(r.name);
            }
        }

        Ok(role_names)
    }

    /// Mengambil daftar semua nama Permission yang dimiliki oleh sebuah model (langsung maupun via Role).
    /// Jika model memiliki role "admin", otomatis mengembalikan semua permission yang ada di sistem.
    pub async fn get_all_permissions_for(
        db: &DatabaseConnection,
        model_type: &str,
        model_id: i32,
    ) -> Result<Vec<String>, DbErr> {
        // Cek apakah user adalah admin
        let roles = Self::get_all_roles_for(db, model_type, model_id).await?;
        if roles.iter().any(|r| r == "admin") {
            let all_perms = permissions::Entity::find().all(db).await?;
            return Ok(all_perms.into_iter().map(|p| p.name).collect());
        }

        let mut perm_ids = std::collections::HashSet::new();

        // 1. Izin langsung
        let direct_perms = model_has_permissions::Entity::find()
            .filter(model_has_permissions::Column::ModelType.eq(model_type))
            .filter(model_has_permissions::Column::ModelId.eq(model_id))
            .all(db)
            .await?;

        for dp in direct_perms {
            perm_ids.insert(dp.permission_id);
        }

        // 2. Izin dari peran (roles)
        let assigned_roles = model_has_roles::Entity::find()
            .filter(model_has_roles::Column::ModelType.eq(model_type))
            .filter(model_has_roles::Column::ModelId.eq(model_id))
            .all(db)
            .await?;

        for rp in assigned_roles {
            let role_perms = role_has_permissions::Entity::find()
                .filter(role_has_permissions::Column::RoleId.eq(rp.role_id))
                .all(db)
                .await?;

            for rp_pivot in role_perms {
                perm_ids.insert(rp_pivot.permission_id);
            }
        }

        let mut perm_names = Vec::new();
        for pid in perm_ids {
            if let Some(p) = permissions::Entity::find_by_id(pid).one(db).await? {
                perm_names.push(p.name);
            }
        }

        Ok(perm_names)
    }
}
