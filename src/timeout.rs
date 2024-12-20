use std::time::Duration;
use tokio::time::timeout;

pub struct Timeout {
    max_duration: Duration,
}

impl Timeout {
    fn new(max_duration: Duration) -> Self {
        Self { max_duration }
    }

    async fn call<F, T, E>(&self, func: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: From<&'static str> + Send + 'static,
    {
        let func = tokio::task::spawn_blocking(func);
        match timeout(self.max_duration, func).await {
            Ok(result) => result.unwrap(),
            Err(_) => Err("Task timed out".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
async fn test_timeout() {
    let time_limiter = Arc::new(Timeout::new(Duration::from_secs(2)));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let time_limiter = time_limiter.clone();
            tokio::spawn(async move {
                let task = move || -> Result<(), &'static str> {
                    println!("Task {} is running", i);
                    std::thread::sleep(Duration::from_secs(3));
                    println!("Task {} is done", i);
                    Ok(())
                };

                match time_limiter.call(task).await {
                    Ok(_) => println!("Task {} succeeded", i),
                    Err(err) => println!("Task {} failed with error: {}", i, err),
                }
            })
        })
        .collect();

    futures::future::join_all(handles).await;
}
}

