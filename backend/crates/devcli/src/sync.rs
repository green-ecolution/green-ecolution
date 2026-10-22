use std::sync::{Mutex, MutexGuard, PoisonError};

/// Every critical section only swaps whole values, so a lock poisoned by a panic is still consistent.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
