use std::sync::{Mutex, MutexGuard};

/// Poison-tolerant locking.
///
/// A panic while a lock is held marks the `Mutex` as poisoned and every later
/// `lock().unwrap()` would panic too, taking the whole command (or app) down.
/// The guarded data here is plain cache/history state, so recovering the inner
/// value is always preferable to crashing.
pub trait LockExt<T> {
    fn lock_or_recover(&self) -> MutexGuard<'_, T>;
}

impl<T> LockExt<T> for Mutex<T> {
    fn lock_or_recover(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|poisoned| {
            eprintln!("Warning: recovering from a poisoned mutex");
            poisoned.into_inner()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn recovers_poisoned_mutex() {
        let m = Arc::new(Mutex::new(5));
        let m2 = Arc::clone(&m);
        let _ = std::thread::spawn(move || {
            let _g = m2.lock().unwrap();
            panic!("poison it");
        })
        .join();
        assert!(m.is_poisoned());
        assert_eq!(*m.lock_or_recover(), 5);
    }
}
