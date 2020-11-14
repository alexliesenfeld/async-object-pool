use async_std::sync::{Condvar, Mutex};

pub struct Pool<T, F> where F: Fn() -> T {
    sync_tuple: Mutex<(Vec<T>, usize)>,
    condvar: Condvar,
    create_max: usize,
    creator_fn: F,
}

impl<T, F> Pool<T, F> where F: Fn() -> T {
    pub fn new(create_max: usize, creator_fn: F) -> Self
    where
        F: Fn() -> T,
    {
        Self {
            sync_tuple: Mutex::new((Vec::new(), 0)),
            condvar: Condvar::new(),
            creator_fn,
            create_max,
        }
    }

    pub async fn put(&self, item: T) {
        let mut lock_guard = (&self.sync_tuple).lock().await;
        (*lock_guard).0.push(item);
        self.condvar.notify_one();
    }

    pub async fn take(&self) -> T {
        let mut lock_guard = (&self.sync_tuple).lock().await;

        while (*lock_guard).0.is_empty() && (*lock_guard).1 == self.create_max {
            lock_guard = self.condvar.wait(lock_guard).await;
        }

        if (*lock_guard).1 < self.create_max {
            let elem = (self.creator_fn)();
            (*lock_guard).0.push(elem);
            (*lock_guard).1 += 1;
        }

        return (*lock_guard).0.remove(0);
    }
}
