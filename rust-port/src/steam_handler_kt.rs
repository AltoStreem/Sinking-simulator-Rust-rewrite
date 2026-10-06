//! Kotlin top-level helpers from `SteamHandlerKt.java`.

use crate::steam_handler::SteamHandler;

pub(crate) fn with_steam<F>(handler: &SteamHandler, callback: F)
where
    F: FnOnce(),
{
    if handler.is_running() { callback(); }
}

pub(crate) fn with_steam_or_later<F>(handler: &mut SteamHandler, callback: F)
where
    F: FnOnce() + Send + 'static,
{
    handler.on_init(callback);
}

pub(crate) fn with_steam_or_default<T, F>(handler: &SteamHandler, default: T, callback: F) -> T
where
    F: FnOnce() -> T,
{
    if handler.is_running() { callback() } else { default }
}

pub(crate) fn assert_steam_call(valid: bool, failure: Option<&str>) {
    debug_assert!(valid, "Steam call failed: {}", failure.unwrap_or("unknown failure"));
}
