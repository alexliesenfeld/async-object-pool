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


#[cfg(test)]
mod tests {
    use crate::Pool;

    #[async_std::test]
    async fn usage_example() -> std::io::Result<()> {
        // Create a new pool that will allow to create at most 100 items
        let pool = Pool::new(100);

        // Take an item from the pool or create a new item if the pool is empty
        // but the maximum number of pooled items was not created yet.
        // This will asynchronously block execution until an item can be returned.
        let item = pool.take_or_create(|| String::from("hello")).await;

        // Use your item
        println!("{}", item);

        // After using the item, put it back into the pool so it can be reused elsewhere
        pool.put(item).await;

        Ok(())
    }
}