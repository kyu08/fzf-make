use std::sync::Mutex;

static PANIC_INFO: Mutex<Option<(String, String)>> = Mutex::new(None);

pub fn set_panic_info(location: String, message: String) {
    if let Ok(mut info) = PANIC_INFO.lock() {
        *info = Some((location, message));
    }
}

pub fn get_panic_info() -> Option<(String, String)> {
    if let Ok(info) = PANIC_INFO.lock() {
        info.clone()
    } else {
        None
    }
}
