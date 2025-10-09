use colored::*;
pub fn print_banner() {
    println!("{}", "========================================".bright_blue().bold());
    println!("{}", " 🚀  Starting ReAuth Application ".bright_green().bold());
    println!("{}", "========================================".bright_blue().bold());

    println!(
        "{} {}",
        "📦 Version:".yellow().bold(),
        env!("CARGO_PKG_VERSION").white()
    );
    println!(
        "{} {}",
        "🏷️  Name:".yellow().bold(),
        env!("CARGO_PKG_NAME").white()
    );
}
