mod controller;
use futures::FutureExt;
use std::panic::AssertUnwindSafe;
mod error;
mod file;
mod model;
use error::panic_info;
mod usecase;

#[tokio::main]
async fn main() {
    // Set panic hook to capture and output backtrace to the debug file.
    std::panic::set_hook(Box::new(|panic_info| {
        use colored::Colorize;

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic message".to_string()
        };

        // Save panic info for later retrieval (for TUI panics)
        panic_info::set_panic_info(location.clone(), message.clone());

        // Display panic information
        eprintln!("{}", format!("thread 'main' panicked at {}", location).red());
        eprintln!("{}", message.red());

        #[cfg(debug_assertions)]
        {
            use time::OffsetDateTime;

            let backtrace = std::backtrace::Backtrace::force_capture();
            let timestamp = OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| "unknown".to_string());

            let debug_info = format!(
                "=== Panic Information ===\nTimestamp: {}\nMessage: {}\nLocation: {}\n\nBacktrace:\n{}",
                timestamp, message, location, backtrace
            );
            let _ = model::file_util::write_debug_info_to_file(&debug_info);
            eprintln!("\n{}", "Panic details have been appended to `debug_info.txt`".red());
        }
    }));

    // ref: https://zenn.dev/techno_tanoc/articles/4c207397df3ab0#assertunwindsafe
    let res = AssertUnwindSafe(controller::controller_main::run())
        .catch_unwind()
        .await;

    if res.is_err() {
        std::process::exit(1);
    }
}
