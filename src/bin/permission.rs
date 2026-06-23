use rustbasic_core::colored::Colorize;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "install" => rustbasic_permission::scaffolding::make_permission_scaffolding(),
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
