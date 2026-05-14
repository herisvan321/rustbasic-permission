use sea_orm::{ConnectionTrait, DatabaseConnection, Iden, DbErr};
use sea_orm::sea_query::{self, Table, ColumnDef, Expr};

#[derive(Iden)]
pub enum Roles {
    Table,
    Id,
    Name,
    GuardName,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum Permissions {
    Table,
    Id,
    Name,
    GuardName,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum ModelHasRoles {
    Table,
    Id,
    RoleId,
    ModelType,
    ModelId,
}

#[derive(Iden)]
pub enum ModelHasPermissions {
    Table,
    Id,
    PermissionId,
    ModelType,
    ModelId,
}

#[derive(Iden)]
pub enum RoleHasPermissions {
    Table,
    Id,
    PermissionId,
    RoleId,
}

/// Membuat seluruh tabel RBAC secara otomatis jika belum ada di database.
pub async fn init_permission_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    let builder = db.get_database_backend();

    // 1. Table `roles`
    let roles_table = Table::create()
        .table(Roles::Table)
        .if_not_exists()
        .col(ColumnDef::new(Roles::Id).integer().not_null().auto_increment().primary_key())
        .col(ColumnDef::new(Roles::Name).string().not_null().unique_key())
        .col(ColumnDef::new(Roles::GuardName).string().not_null().default("web"))
        .col(ColumnDef::new(Roles::CreatedAt).timestamp().default(Expr::current_timestamp()).not_null())
        .col(ColumnDef::new(Roles::UpdatedAt).timestamp().default(Expr::current_timestamp()).not_null())
        .to_owned();

    db.execute(builder.build(&roles_table)).await?;

    // 2. Table `permissions`
    let permissions_table = Table::create()
        .table(Permissions::Table)
        .if_not_exists()
        .col(ColumnDef::new(Permissions::Id).integer().not_null().auto_increment().primary_key())
        .col(ColumnDef::new(Permissions::Name).string().not_null().unique_key())
        .col(ColumnDef::new(Permissions::GuardName).string().not_null().default("web"))
        .col(ColumnDef::new(Permissions::CreatedAt).timestamp().default(Expr::current_timestamp()).not_null())
        .col(ColumnDef::new(Permissions::UpdatedAt).timestamp().default(Expr::current_timestamp()).not_null())
        .to_owned();

    db.execute(builder.build(&permissions_table)).await?;

    // 3. Table `model_has_roles`
    let model_has_roles_table = Table::create()
        .table(ModelHasRoles::Table)
        .if_not_exists()
        .col(ColumnDef::new(ModelHasRoles::Id).integer().not_null().auto_increment().primary_key())
        .col(ColumnDef::new(ModelHasRoles::RoleId).integer().not_null())
        .col(ColumnDef::new(ModelHasRoles::ModelType).string().not_null())
        .col(ColumnDef::new(ModelHasRoles::ModelId).integer().not_null())
        .to_owned();

    db.execute(builder.build(&model_has_roles_table)).await?;

    // 4. Table `model_has_permissions`
    let model_has_permissions_table = Table::create()
        .table(ModelHasPermissions::Table)
        .if_not_exists()
        .col(ColumnDef::new(ModelHasPermissions::Id).integer().not_null().auto_increment().primary_key())
        .col(ColumnDef::new(ModelHasPermissions::PermissionId).integer().not_null())
        .col(ColumnDef::new(ModelHasPermissions::ModelType).string().not_null())
        .col(ColumnDef::new(ModelHasPermissions::ModelId).integer().not_null())
        .to_owned();

    db.execute(builder.build(&model_has_permissions_table)).await?;

    // 5. Table `role_has_permissions`
    let role_has_permissions_table = Table::create()
        .table(RoleHasPermissions::Table)
        .if_not_exists()
        .col(ColumnDef::new(RoleHasPermissions::Id).integer().not_null().auto_increment().primary_key())
        .col(ColumnDef::new(RoleHasPermissions::PermissionId).integer().not_null())
        .col(ColumnDef::new(RoleHasPermissions::RoleId).integer().not_null())
        .to_owned();

    db.execute(builder.build(&role_has_permissions_table)).await?;

    tracing::info!("Tabel-tabel rustbasic-permission berhasil diinisialisasi.");
    Ok(())
}
