use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::rngs::OsRng;
use redis_pool::RedisPool;
use redis_pool::SingleRedisPool;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;

// Appstate that needs to be shared between endpoints
#[derive(Clone)]
pub struct AppState {
    pub postgres: Pool<Postgres>,
    pub redis: SingleRedisPool,
}

const BAD_DOT_ENV: &str = "Bad .env file";

// In case there is no administrator account, a default one will be made
pub async fn create_default_account(pool: &Pool<Postgres>) {
    let salt = SaltString::generate(&mut OsRng);
    let password = Argon2::default()
        .hash_password(
            env::var("DEFAULT_ADMIN_PASSWORD")
                .expect(BAD_DOT_ENV)
                .as_bytes(),
            &salt,
        )
        .unwrap();

    let _ = sqlx::query!(
        "INSERT INTO users (username, pass, token, tok_expire) VALUES ('Admin', $1, '', NOW())",
        password.to_string(),
    )
    .execute(pool)
    .await;
}

pub async fn init_db() -> AppState {
    dotenvy::dotenv().expect(".env file error");

    let pool = PgPoolOptions::new()
        .max_connections(
            (env::var("MAX_DB_POOL_CONNECTIONS").expect(BAD_DOT_ENV))
                .parse()
                .unwrap(),
        )
        .connect(&env::var("DATABASE_URL").expect(BAD_DOT_ENV))
        .await
        .unwrap();

    create_default_account(&pool).await;

    let client = redis::Client::open(env::var("REDIS_ADDRESS").expect(BAD_DOT_ENV).as_str())
        .expect("Error while testing the connection");

    AppState {
        postgres: pool,
        redis: RedisPool::from(client),
    }
}
