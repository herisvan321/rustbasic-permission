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
r#"use rustbasic_core::{{Schema, SchemaManager, MigrationTrait, DbErr}};
use rustbasic_core::async_trait;

pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {{
    fn name(&self) -> &str {{
        "{migration_name}"
    }}

    async fn up<'a>(&self, manager: &'a SchemaManager<'a>) -> Result<(), DbErr> {{
        // 1. Table roles
        Schema::create(manager, "roles", |table| {{
            table.string("name").unique().not_null();
            table.string("guard_name").default("web").not_null();
        }}).await?;

        // 2. Table permissions
        Schema::create(manager, "permissions", |table| {{
            table.string("name").unique().not_null();
            table.string("guard_name").default("web").not_null();
        }}).await?;

        // 3. Table model_has_roles
        Schema::create(manager, "model_has_roles", |table| {{
            table.no_timestamps();
            table.integer("role_id").not_null();
            table.string("model_type").not_null();
            table.integer("model_id").not_null();
        }}).await?;

        // 4. Table model_has_permissions
        Schema::create(manager, "model_has_permissions", |table| {{
            table.no_timestamps();
            table.integer("permission_id").not_null();
            table.string("model_type").not_null();
            table.integer("model_id").not_null();
        }}).await?;

        // 5. Table role_has_permissions
        Schema::create(manager, "role_has_permissions", |table| {{
            table.no_timestamps();
            table.integer("permission_id").not_null();
            table.integer("role_id").not_null();
        }}).await?;

        Ok(())
    }}

    async fn down<'a>(&self, manager: &'a SchemaManager<'a>) -> Result<(), DbErr> {{
        Schema::drop(manager, "role_has_permissions").await?;
        Schema::drop(manager, "model_has_permissions").await?;
        Schema::drop(manager, "model_has_roles").await?;
        Schema::drop(manager, "permissions").await?;
        Schema::drop(manager, "roles").await?;
        Ok(())
    }}
}}
"#, migration_name = migration_name);

    fs::write(&migration_path, migration_template).expect("Gagal membuat migration RBAC");
    update_migration_mod_rs(&migration_name);
    println!("   {} Migration dibuat: {}", "📦".blue(), migration_path.cyan());

    // 2. Buat Model
    let models = vec![
        ("role", "roles", "Role", "    pub name: String,\n        pub guard_name: String,"),
        ("permission", "permissions", "Permission", "    pub name: String,\n        pub guard_name: String,"),
        ("model_has_role", "model_has_roles", "ModelHasRole", "    pub role_id: i32,\n        pub model_type: String,\n        pub model_id: i32,"),
        ("model_has_permission", "model_has_permissions", "ModelHasPermission", "    pub permission_id: i32,\n        pub model_type: String,\n        pub model_id: i32,"),
        ("role_has_permission", "role_has_permissions", "RoleHasPermission", "    pub permission_id: i32,\n        pub role_id: i32,"),
    ];

    fs::create_dir_all("src/app/models").ok();

    for (name, table, class_name, fields) in models {
        let file_path = format!("src/app/models/{}.rs", name);
        if !std::path::Path::new(&file_path).exists() {
            let model_template = format!(
r#"use rustbasic_core::model;

model! {{
    table: "{table}",
    {class_name} {{
        pub id: i32,
    {fields}
    }}
}}
"#, table = table, class_name = class_name, fields = fields);
            
            fs::write(&file_path, model_template).expect("Gagal membuat model");
            update_model_mod_rs(class_name, name);
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
    if let Some(pos) = content.find(search_pattern)
        && let Some(insert_pos) = content[pos..].find("        ]") {
        let absolute_insert_pos = pos + insert_pos;
        content.insert_str(absolute_insert_pos, &format!("            Box::new({}::Migration),\n", mod_name));
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
    writeln!(file, "pub use {}::{};", snake_name, class_name).ok();
}
