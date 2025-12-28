//! CADDY - Enterprise CAD System
//!
//! Main entry point for the CADDY application.
//!
//! This application integrates:
//! - Core math and geometry primitives
//! - 2D and 3D geometric operations
//! - GPU-accelerated rendering with wgpu
//! - Professional CAD user interface with egui
//! - File I/O for DXF and native formats
//! - Command system with undo/redo
//! - Layer management
//! - Selection and transformation tools
//! - Dimensioning and annotations
//! - Parametric constraint solving

use caddy::ui::window::run_app;
use std::panic;

fn main() -> anyhow::Result<()> {
    // Set up panic hook for better error reporting
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("CADDY Fatal Error:");
        eprintln!("{}", panic_info);

        if let Some(location) = panic_info.location() {
            eprintln!("Location: {}:{}:{}",
                location.file(),
                location.line(),
                location.column());
        }

        // Log backtrace if available
        eprintln!("\nPlease report this error at: https://github.com/caddy-cad/caddy/issues");
    }));

    // Initialize logging with configurable level
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string());

    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or(log_level)
    ).init();

    // Print startup banner
    log::info!("═══════════════════════════════════════════════════════════");
    log::info!("  CADDY - Enterprise Computer-Aided Design System");
    log::info!("  Version: {}", caddy::VERSION);
    log::info!("  Built with Rust for performance and reliability");
    log::info!("═══════════════════════════════════════════════════════════");

    // Log system information
    log::info!("System Information:");
    log::info!("  Platform: {}", std::env::consts::OS);
    log::info!("  Architecture: {}", std::env::consts::ARCH);

    // Check for required features
    log::debug!("Checking system capabilities...");

    // Run the application
    log::info!("Launching application window...");
    let result = run_app();

    match &result {
        Ok(_) => {
            log::info!("═══════════════════════════════════════════════════════════");
            log::info!("  CADDY shutdown complete");
            log::info!("═══════════════════════════════════════════════════════════");
        }
        Err(e) => {
            log::error!("Application error: {}", e);
            log::error!("Please check the logs for more information");
        }
    }

    result
}
