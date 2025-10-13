mod controller;
use futures::FutureExt;
use std::panic::AssertUnwindSafe;
mod err;
mod file;
mod model;
mod usecase;

#[tokio::main]
async fn main() {
    // Set panic hook to capture backtrace in debug builds
    #[cfg(debug_assertions)]
    {
        std::panic::set_hook(Box::new(|panic_info| {
            use std::time::SystemTime;

            let backtrace = std::backtrace::Backtrace::force_capture();
            let location = panic_info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_else(|| "unknown".to_string());
            let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic message".to_string()
            };

            let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(duration) => {
                    let secs = duration.as_secs();
                    let millis = duration.subsec_millis();
                    format!("{}.{:03}", secs, millis)
                }
                Err(_) => "unknown".to_string(),
            };

            let debug_info = format!(
                "=== Panic Information ===\nTimestamp: {} (UNIX epoch)\nMessage: {}\nLocation: {}\n\nBacktrace:\n{}\n",
                timestamp, message, location, backtrace
            );
            let _ = model::file_util::write_debug_info_to_file(&debug_info);
            eprintln!("Panic details have been written to debug_info.txt");
        }));
    }

    // ref: https://zenn.dev/techno_tanoc/articles/4c207397df3ab0#assertunwindsafe
    let res = AssertUnwindSafe(controller::controller_main::run())
        .catch_unwind()
        .await;

    if let Err(e) = res {
        println!("{}", err::any_to_string::any_to_string(&*e));
        std::process::exit(1);
    }
}
