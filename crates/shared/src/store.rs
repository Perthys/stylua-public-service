use redis::aio::ConnectionManager;
use uuid::Uuid;

pub type Store = ConnectionManager;

const QUEUE_KEY: &str = "format:queue";
const PERMITS_KEY: &str = "format:permits";
const SESSION_TTL: usize = 300;

const ACQUIRE_SCRIPT: &str = include_str!("scripts/acquire-script.lua");

fn session_key(session_id: &str) -> String {
    format!("session:{session_id}")
}

pub async fn connect(url: &str) -> redis::RedisResult<Store> {
    let client = redis::Client::open(url)?;
    ConnectionManager::new(client).await
}

pub async fn enqueue(connection: &mut Store, job: &str) -> redis::RedisResult<()> {
    redis::cmd("LPUSH")
        .arg(QUEUE_KEY)
        .arg(job)
        .query_async(connection)
        .await
}

pub async fn dequeue(connection: &mut Store) -> redis::RedisResult<Option<String>> {
    redis::cmd("RPOP")
        .arg(QUEUE_KEY)
        .query_async(connection)
        .await
}

pub async fn queue_len(connection: &mut Store) -> redis::RedisResult<usize> {
    redis::cmd("LLEN")
        .arg(QUEUE_KEY)
        .query_async(connection)
        .await
}

pub async fn acquire(
    connection: &mut Store,
    limit: usize,
    ttl: usize,
) -> redis::RedisResult<Option<String>> {
    let permit_id = Uuid::new_v4().to_string();
    let admitted: bool = redis::Script::new(ACQUIRE_SCRIPT)
        .key(PERMITS_KEY)
        .arg(ttl as u64)
        .arg(limit as u64)
        .arg(&permit_id)
        .invoke_async(connection)
        .await?;

    if admitted {
        Ok(Some(permit_id))
    } else {
        Ok(None)
    }
}

pub async fn release(connection: &mut Store, permit_id: &str) -> redis::RedisResult<()> {
    redis::cmd("ZREM")
        .arg(PERMITS_KEY)
        .arg(permit_id)
        .query_async(connection)
        .await
}

pub async fn get_session(
    connection: &mut Store,
    session_id: &str,
) -> redis::RedisResult<Option<String>> {
    redis::cmd("GET")
        .arg(session_key(session_id))
        .query_async(connection)
        .await
}

pub async fn set_session(
    connection: &mut Store,
    session_id: &str,
    data: &str,
) -> redis::RedisResult<()> {
    redis::cmd("SET")
        .arg(session_key(session_id))
        .arg(data)
        .arg("EX")
        .arg(SESSION_TTL)
        .query_async(connection)
        .await
}

pub async fn delete_session(connection: &mut Store, session_id: &str) -> redis::RedisResult<()> {
    redis::cmd("DEL")
        .arg(session_key(session_id))
        .query_async(connection)
        .await
}
