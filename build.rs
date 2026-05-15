use std::fs;
use std::path::PathBuf;
use std::env;

fn main() {
    // Hanya jalankan jika kita tidak sedang dalam proses rilis atau docs
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    // Ambil direktori kerja saat ini. 
    // Saat 'cargo build' dijalankan di proyek utama (misal: cobasaja), 
    // PWD biasanya menunjuk ke root proyek tersebut.
    let project_root = match env::var("PWD") {
        Ok(pwd) => PathBuf::from(pwd),
        Err(_) => match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return,
        },
    };

    // Pastikan ini adalah proyek RustBasic (minimal ada Cargo.toml)
    if !project_root.join("Cargo.toml").exists() {
        return;
    }

    // JANGAN lakukan scaffolding jika kita sedang men-debug paket rustbasic-permission itu sendiri
    if project_root.join("src/bin/permission.rs").exists() {
        return;
    }

    println!("cargo:warning=🔐 rustbasic-permission: Menyiapkan scaffolding otomatis...");

    // 1. Buat Migration
    let migrations_dir = project_root.join("database/migrations");
    fs::create_dir_all(&migrations_dir).ok();

    // Cek apakah sudah ada migrasi RBAC (hindari duplikasi)
    let existing_migrations = fs::read_dir(&migrations_dir)
        .map(|dir| dir.filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().contains("create_rbac_tables")))
        .unwrap_or(false);

    if !existing_migrations {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let migration_name = format!("m{}_create_rbac_tables", timestamp);
        let migration_path = migrations_dir.join(format!("{}.rs", migration_name));

        let migration_template = format!(
r#"use sea_orm_migration::prelude::*;
use async_trait::async_trait;

#[derive(Iden)]
pub enum Roles {{
    Table, Id, Name, GuardName, CreatedAt, UpdatedAt,
}}

#[derive(Iden)]
pub enum Permissions {{
    Table, Id, Name, GuardName, CreatedAt, UpdatedAt,
}}

#[derive(Iden)]
pub enum ModelHasRoles {{
    Table, Id, RoleId, ModelType, ModelId,
}}

#[derive(Iden)]
pub enum ModelHasPermissions {{
    Table, Id, PermissionId, ModelType, ModelId,
}}

#[derive(Iden)]
pub enum RoleHasPermissions {{
    Table, Id, PermissionId, RoleId,
}}

#[derive(Iden)]
pub struct Migration;

impl MigrationName for Migration {{
    fn name(&self) -> &str {{
        "{migration_name}"
    }}
}}

#[async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager.create_table(Table::create().table(Roles::Table).if_not_exists().col(ColumnDef::new(Roles::Id).integer().not_null().auto_increment().primary_key()).col(ColumnDef::new(Roles::Name).string().not_null().unique_key()).col(ColumnDef::new(Roles::GuardName).string().not_null().default("web")).col(ColumnDef::new(Roles::CreatedAt).date_time().default(Expr::current_timestamp())).col(ColumnDef::new(Roles::UpdatedAt).date_time().default(Expr::current_timestamp())).to_owned()).await?;
        manager.create_table(Table::create().table(Permissions::Table).if_not_exists().col(ColumnDef::new(Permissions::Id).integer().not_null().auto_increment().primary_key()).col(ColumnDef::new(Permissions::Name).string().not_null().unique_key()).col(ColumnDef::new(Permissions::GuardName).string().not_null().default("web")).col(ColumnDef::new(Permissions::CreatedAt).date_time().default(Expr::current_timestamp())).col(ColumnDef::new(Permissions::UpdatedAt).date_time().default(Expr::current_timestamp())).to_owned()).await?;
        manager.create_table(Table::create().table(ModelHasRoles::Table).if_not_exists().col(ColumnDef::new(ModelHasRoles::Id).integer().not_null().auto_increment().primary_key()).col(ColumnDef::new(ModelHasRoles::RoleId).integer().not_null()).col(ColumnDef::new(ModelHasRoles::ModelType).string().not_null()).col(ColumnDef::new(ModelHasRoles::ModelId).integer().not_null()).to_owned()).await?;
        manager.create_table(Table::create().table(ModelHasPermissions::Table).if_not_exists().col(ColumnDef::new(ModelHasPermissions::Id).integer().not_null().auto_increment().primary_key()).col(ColumnDef::new(ModelHasPermissions::PermissionId).integer().not_null()).col(ColumnDef::new(ModelHasPermissions::ModelType).string().not_null()).col(ColumnDef::new(ModelHasPermissions::ModelId).integer().not_null()).to_owned()).await?;
        manager.create_table(Table::create().table(RoleHasPermissions::Table).if_not_exists().col(ColumnDef::new(RoleHasPermissions::Id).integer().not_null().auto_increment().primary_key()).col(ColumnDef::new(RoleHasPermissions::PermissionId).integer().not_null()).col(ColumnDef::new(RoleHasPermissions::RoleId).integer().not_null()).to_owned()).await?;
        Ok(())
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager.drop_table(Table::drop().table(RoleHasPermissions::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(ModelHasPermissions::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(ModelHasRoles::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Permissions::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Roles::Table).to_owned()).await?;
        Ok(())
    }}
}}
"#, migration_name = migration_name);

        if fs::write(&migration_path, migration_template).is_ok() {
            update_migration_mod_rs(&project_root, &migration_name);
        }
    }

    // 2. Buat Model
    let models = vec![
        ("role", "roles", "    pub name: String,\n    pub guard_name: String,"),
        ("permission", "permissions", "    pub name: String,\n    pub guard_name: String,"),
        ("model_has_role", "model_has_roles", "    pub role_id: i32,\n    pub model_type: String,\n    pub model_id: i32,"),
        ("model_has_permission", "model_has_permissions", "    pub permission_id: i32,\n    pub model_type: String,\n    pub model_id: i32,"),
        ("role_has_permission", "role_has_permissions", "    pub permission_id: i32,\n    pub role_id: i32,"),
    ];

    let models_dir = project_root.join("src/app/models");
    fs::create_dir_all(&models_dir).ok();

    for (name, table, fields) in models {
        let file_path = models_dir.join(format!("{}.rs", name));
        if !file_path.exists() {
            let model_template = format!(
r#"use rustbasic_core::sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{table}")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i32,
{fields}
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}
"#, table = table, fields = fields);

            if fs::write(&file_path, model_template).is_ok() {
                update_model_mod_rs(&project_root, &to_pascal_case(name), name);
            }
        }
    }
}

fn update_migration_mod_rs(project_root: &std::path::Path, mod_name: &str) {
    let mod_path = project_root.join("database/migrations/mod.rs");
    if !mod_path.exists() { return; }

    let mut content = fs::read_to_string(&mod_path).unwrap_or_default();

    if !content.contains(&format!("pub mod {};", mod_name)) {
        content.push_str(&format!("\npub mod {};\n", mod_name));
    }

    let search_pattern = "fn migrations() -> Vec<Box<dyn MigrationTrait>> {";
    if let Some(pos) = content.find(search_pattern) {
        if let Some(insert_pos) = content[pos..].find("        ]") {
            content.insert_str(pos + insert_pos, &format!("            Box::new({}::Migration),\n", mod_name));
        }
    }

    fs::write(mod_path, content).ok();
}

fn update_model_mod_rs(project_root: &std::path::Path, class_name: &str, snake_name: &str) {
    let mod_path = project_root.join("src/app/models/mod.rs");
    if !mod_path.exists() { return; }

    let content = fs::read_to_string(&mod_path).unwrap_or_default();
    if content.contains(&format!("pub mod {};", snake_name)) {
        return;
    }

    let mut file = fs::OpenOptions::new().append(true).open(mod_path).unwrap();
    use std::io::Write;
    writeln!(file, "pub mod {};", snake_name).ok();
    writeln!(file, "pub use {}::Entity as {};", snake_name, class_name).ok();
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' { capitalize_next = true; }
        else if capitalize_next { result.push(c.to_ascii_uppercase()); capitalize_next = false; }
        else { result.push(c); }
    }
    result
}
