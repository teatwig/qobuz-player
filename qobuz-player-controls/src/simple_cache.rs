use time::Duration;
use tokio::{sync::RwLock, time::Instant};

#[derive(Debug)]
pub(crate) struct SimpleCache<T> {
    value: RwLock<Option<T>>,
    ttl: Duration,
    created: RwLock<Option<Instant>>,
}

impl<T> SimpleCache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            value: RwLock::new(None),
            ttl,
            created: RwLock::new(None),
        }
    }

    pub async fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        if self.valid().await {
            self.value.read().await.clone()
        } else {
            None
        }
    }

    pub async fn set(&self, value: T) {
        *self.value.write().await = Some(value);
        *self.created.write().await = Some(Instant::now());
    }

    pub async fn clear(&self) {
        *self.value.write().await = None;
        *self.created.write().await = None;
    }

    async fn valid(&self) -> bool {
        match *self.created.read().await {
            Some(created) => created.elapsed() < self.ttl,
            None => false,
        }
    }
}
