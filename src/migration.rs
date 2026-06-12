use rustbasic_core::sql::{AnyPool, Error};
use rustbasic_core::schema::{Schema, SchemaManager};

/// Membuat seluruh tabel RBAC secara otomatis jika belum ada di database.
pub async fn init_permission_tables(pool: &AnyPool) -> Result<(), Error> {
    let manager = SchemaManager::new(pool);

    // 1. Table `roles`
    Schema::create(&manager, "roles", |table| {
        table.string("name").unique().not_null();
        table.string("guard_name").default("web").not_null();
    }).await?;

    // 2. Table `permissions`
    Schema::create(&manager, "permissions", |table| {
        table.string("name").unique().not_null();
        table.string("guard_name").default("web").not_null();
    }).await?;

    // 3. Table `model_has_roles`
    Schema::create(&manager, "model_has_roles", |table| {
        table.no_timestamps();
        table.integer("role_id").not_null();
        table.string("model_type").not_null();
        table.integer("model_id").not_null();
    }).await?;

    // 4. Table `model_has_permissions`
    Schema::create(&manager, "model_has_permissions", |table| {
        table.no_timestamps();
        table.integer("permission_id").not_null();
        table.string("model_type").not_null();
        table.integer("model_id").not_null();
    }).await?;

    // 5. Table `role_has_permissions`
    Schema::create(&manager, "role_has_permissions", |table| {
        table.no_timestamps();
        table.integer("permission_id").not_null();
        table.integer("role_id").not_null();
    }).await?;

    rustbasic_core::tracing::info!("Tabel-tabel rustbasic-permission berhasil diinisialisasi.");
    Ok(())
}
