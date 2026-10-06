//! Offline-safe Rust counterpart of `SteamHandler.java`.
//!
//! The original class owns the Steam lifetime and queues callbacks until the
//! API has initialized.  The port keeps that contract behind an injectable
//! runtime so the simulator remains playable when Steam is unavailable.

use std::collections::VecDeque;

pub(crate) struct SteamHandler {
    running: bool,
    failed: bool,
    callbacks: VecDeque<Box<dyn FnOnce() + Send>>,
}

impl Default for SteamHandler {
    fn default() -> Self {
        Self {
            running: false,
            failed: false,
            callbacks: VecDeque::new(),
        }
    }
}

impl SteamHandler {
    pub(crate) fn is_running(&self) -> bool { self.running }
    pub(crate) fn failed(&self) -> bool { self.failed }

    pub(crate) fn init(&mut self, running: bool) {
        self.running = running;
        self.failed = !running;
        if running {
            while let Some(callback) = self.callbacks.pop_front() {
                callback();
            }
        } else {
            self.callbacks.clear();
        }
    }

    pub(crate) fn on_init<F>(&mut self, callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if self.running { callback(); }
        else if !self.failed { self.callbacks.push_back(Box::new(callback)); }
    }

    pub(crate) fn update(&self) {
        // SteamAPI.runCallbacks() is intentionally an injected backend hook;
        // the no-Steam build has no native library to call here.
    }

    pub(crate) fn free(&mut self) {
        self.running = false;
        self.callbacks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn callbacks_wait_until_initialization() {
        let mut handler = SteamHandler::default();
        let called = Arc::new(Mutex::new(false));
        let target = Arc::clone(&called);
        handler.on_init(move || *target.lock().unwrap() = true);
        assert!(!*called.lock().unwrap());
        handler.init(true);
        assert!(*called.lock().unwrap());
    }
}
