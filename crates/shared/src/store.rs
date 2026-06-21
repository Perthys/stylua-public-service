use redis::aio::ConnectionManager;

pub type Store = ConnectionManager;

pub async fn connect(url: &str) -> redis::RedisResult<Store> {
    let client = redis::Client::open(url)?;
    ConnectionManager::new(client).await
}
