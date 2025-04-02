use dct::AppState;
use redis_pool::RedisPool;
use sqlx::postgres::PgPoolOptions;
use std::env;

pub async fn init_db() -> AppState {
    dotenvy::dotenv().unwrap();
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set as an environment variable");

    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .unwrap();

    dct::create_default_account(&pool).await;

    let client = RedisPool::from(
        redis::Client::open(env::var("REDIS_ADDRESS").unwrap().as_str())
            .expect("Error while testing the connection"),
    );

    let mut conn = client.aquire().await.unwrap();

    let _ = redis::cmd("flushdb")
        .query_async::<_, Vec<u8>>(&mut conn)
        .await;

    AppState {
        postgres: pool,
        redis: client,
    }
}

// pub async fn flush_redis_cache(state: &AppState) {
//     let mut conn = state.redis.aquire().await.unwrap();
//     redis::cmd("flushdb")
//         .query_async::<_, Vec<u8>>(&mut conn)
//         .await;
// }
