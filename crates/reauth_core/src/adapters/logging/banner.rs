use crate::config::Settings;
use colored::*;
use std::env;

pub fn print_banner(settings: &Settings) {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let target = format!("{}/{}", env::consts::OS, env::consts::ARCH);
    let pid = std::process::id();
    let ui_mode = if cfg!(feature = "embed-ui") {
        "embedded"
    } else {
        "external"
    };
    let config_path = Settings::resolve_config_watch_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "none".to_string());
    let exe_path = env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let logo = [
        "ooooooooo.                   .o.                       .   oooo       ",
        "`888   `Y88.                .888.                    .o8   `888       ",
        " 888   .d88'  .ooooo.      .8\"888.     oooo  oooo  .o888oo  888 .oo.  ",
        " 888ooo88P'  d88' `88b    .8' `888.    `888  `888    888    888P\"Y88b ",
        " 888`88b.    888ooo888   .88ooo8888.    888   888    888    888   888 ",
        " 888  `88b.  888    .o  .8'     `888.   888   888    888 .  888   888 ",
        "o888o  o888o `Y8bod8P' o88o     o8888o  `V88V\"V8P'   \"888\" o888o o888o",
    ];

    let steps = (logo.len().saturating_sub(1)).max(1) as f32;
    for (idx, line) in logo.iter().enumerate() {
        let ratio = idx as f32 / steps;
        let g = (255.0 * ratio).round() as u8;
        let color = Color::TrueColor { r: 255, g, b: g };
        println!("{}", line.color(color).bold());
    }

    println!("{} {}", "Name:".yellow().bold(), name.white());
    println!("{} {}", "Version:".yellow().bold(), version.white());
    println!("{} {}", "Target:".yellow().bold(), target.white());
    println!("{} {}", "PID:".yellow().bold(), pid.to_string().white());
    println!(
        "{} {}",
        "Public URL:".yellow().bold(),
        settings.server.public_url.white()
    );
    println!("{} {}", "UI Mode:".yellow().bold(), ui_mode.white());
    println!("{} {}", "Config File:".yellow().bold(), config_path.white());
    println!("{} {}", "Binary:".yellow().bold(), exe_path.white());
}
