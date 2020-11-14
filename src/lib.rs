use async_std::sync::{Condvar, Mutex};

pub struct Pool<T> {
    sync_tuple: Mutex<(Vec<T>, usize)>,
    condvar: Condvar,
    create_max: usize
}

impl<T> Pool<T> {
    pub fn new(create_max: usize) -> Self {
        Self {
            sync_tuple: Mutex::new((Vec::new(), 0)),
            condvar: Condvar::new(),
            create_max,
        }
    }

    pub async fn put(&self, item: T) {
        let mut lock_guard = (&self.sync_tuple).lock().await;
        (*lock_guard).0.push(item);
        self.condvar.notify_one();
    }

    pub async fn take_or_create<F>(&self, creator_fn: F) -> T where
        F: Fn() -> T {
        let mut lock_guard = (&self.sync_tuple).lock().await;

        while (*lock_guard).0.is_empty() && (*lock_guard).1 == self.create_max {
            lock_guard = self.condvar.wait(lock_guard).await;
        }

        if (*lock_guard).1 < self.create_max {
            (*lock_guard).0.push((creator_fn)());
            (*lock_guard).1 += 1;
        }

        return (*lock_guard).0.remove(0);
    }
}
