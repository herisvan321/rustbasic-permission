use std::fs::{self, OpenOptions};
use std::io::Write;
use chrono::Local;
use colored::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "install" => make_permission_scaffolding(),
        _ => {
            println!("{} {}", "❌ Error: Perintah tidak dikenal:".red().bold(), args[1].yellow());
            print_help();
        }
    }
}

fn print_help() {
    println!("\n{}", "🔐 RustBasic Permission CLI".magenta().bold());
    println!("{}", "===========================".magenta());
    println!("{}", "Usage:".bold());
    println!("  rustbasic-permission install    {}", "Scaffold RBAC tables and models into your project".dimmed());
    println!();
}

pub fn make_permission_scaffolding() {
    println!("\n{} {}", "🚀".bold(), "Menyiapkan scaffolding RBAC (Role-Based Access Control)...".magenta().bold());

    // Cek apakah kita berada di root project RustBasic
    if !std::path::Path::new("Cargo.toml").exists() {
        println!("{}", "❌ Error: File Cargo.toml tidak ditemukan. Pastikan Anda menjalankan perintah ini di root proyek.".red().bold());
        return;
    }

    // 1. Buat Migration
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let migration_name = format!("m{}_create_rbac_tables", timestamp);
    let migration_path = format!("database/migrations/{}.rs", migration_name);

    // Pastikan folder migrations ada
    fs::create_dir_all("database/migrations").ok();

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
        // 1. Table roles
        manager.create_table(
            Table::create()
                .table(Roles::Table)
                .if_not_exists()
                .col(ColumnDef::new(Roles::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Roles::Name).string().not_null().unique_key())
                .col(ColumnDef::new(Roles::GuardName).string().not_null().default("web"))
                .col(ColumnDef::new(Roles::CreatedAt).date_time().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Roles::UpdatedAt).date_time().default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        // 2. Table permissions
        manager.create_table(
            Table::create()
                .table(Permissions::Table)
                .if_not_exists()
                .col(ColumnDef::new(Permissions::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Permissions::Name).string().not_null().unique_key())
                .col(ColumnDef::new(Permissions::GuardName).string().not_null().default("web"))
                .col(ColumnDef::new(Permissions::CreatedAt).date_time().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Permissions::UpdatedAt).date_time().default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        // 3. Table model_has_roles
        manager.create_table(
            Table::create()
                .table(ModelHasRoles::Table)
                .if_not_exists()
                .col(ColumnDef::new(ModelHasRoles::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(ModelHasRoles::RoleId).integer().not_null())
                .col(ColumnDef::new(ModelHasRoles::ModelType).string().not_null())
                .col(ColumnDef::new(ModelHasRoles::ModelId).integer().not_null())
                .to_owned(),
        ).await?;

        // 4. Table model_has_permissions
        manager.create_table(
            Table::create()
                .table(ModelHasPermissions::Table)
                .if_not_exists()
                .col(ColumnDef::new(ModelHasPermissions::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(ModelHasPermissions::PermissionId).integer().not_null())
                .col(ColumnDef::new(ModelHasPermissions::ModelType).string().not_null())
                .col(ColumnDef::new(ModelHasPermissions::ModelId).integer().not_null())
                .to_owned(),
        ).await?;

        // 5. Table role_has_permissions
        manager.create_table(
            Table::create()
                .table(RoleHasPermissions::Table)
                .if_not_exists()
                .col(ColumnDef::new(RoleHasPermissions::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(RoleHasPermissions::PermissionId).integer().not_null())
                .col(ColumnDef::new(RoleHasPermissions::RoleId).integer().not_null())
                .to_owned(),
        ).await?;

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

    fs::write(&migration_path, migration_template).expect("Gagal membuat migration RBAC");
    update_migration_mod_rs(&migration_name);
    println!("   {} Migration dibuat: {}", "📦".blue(), migration_path.cyan());

    // 2. Buat Model
    let models = vec![
        ("role", "roles"),
        ("permission", "permissions"),
        ("model_has_role", "model_has_roles"),
        ("model_has_permission", "model_has_permissions"),
        ("role_has_permission", "role_has_permissions"),
    ];

    fs::create_dir_all("src/app/models").ok();

    for (name, table) in models {
        let file_path = format!("src/app/models/{}.rs", name);
        if !std::path::Path::new(&file_path).exists() {
            let fields = match name {
                "role" | "permission" => 
                    "    pub name: String,\n    pub guard_name: String,",
                "model_has_role" =>
                    "    pub role_id: i32,\n    pub model_type: String,\n    pub model_id: i32,",
                "model_has_permission" =>
                    "    pub permission_id: i32,\n    pub model_type: String,\n    pub model_id: i32,",
                "role_has_permission" =>
                    "    pub permission_id: i32,\n    pub role_id: i32,",
                _ => ""
            };

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
            
            fs::write(&file_path, model_template).expect("Gagal membuat model");
            update_model_mod_rs(&to_pascal_case(name), name);
            println!("   {} Model dibuat: {}", "📄".blue(), file_path.cyan());
        }
    }

    println!("\n{} {}", "✅".green(), "Scaffolding RBAC berhasil diselesaikan!".green().bold());
    println!("{} Jalankan '{}' untuk menerapkan tabel ke database.", "💡".yellow(), "rustbasic migrate".cyan());
}

fn update_migration_mod_rs(mod_name: &str) {
    let mod_path = "database/migrations/mod.rs";
    if !std::path::Path::new(mod_path).exists() { return; }

    let mut content = fs::read_to_string(mod_path).unwrap_or_default();

    // Tambahkan mod declaration
    if !content.contains(&format!("pub mod {};", mod_name)) {
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&format!("pub mod {};\n", mod_name));
    }

    // Tambahkan ke list migrations
    let search_pattern = "fn migrations() -> Vec<Box<dyn MigrationTrait>> {";
    if let Some(pos) = content.find(search_pattern) {
        if let Some(insert_pos) = content[pos..].find("        ]") {
            let absolute_insert_pos = pos + insert_pos;
            content.insert_str(absolute_insert_pos, &format!("            Box::new({}::Migration),\n", mod_name));
        }
    }

    fs::write(mod_path, content).ok();
}

fn update_model_mod_rs(class_name: &str, snake_name: &str) {
    let mod_path = "src/app/models/mod.rs";
    if !std::path::Path::new(mod_path).exists() { return; }

    let content = fs::read_to_string(mod_path).unwrap_or_default();

    let mod_line = format!("pub mod {};", snake_name);
    if content.contains(&mod_line) {
        return;
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(mod_path)
        .expect("Gagal membuka models/mod.rs");

    writeln!(file, "{}", mod_line).ok();
    writeln!(file, "pub use {}::Entity as {};", snake_name, class_name).ok();
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}
