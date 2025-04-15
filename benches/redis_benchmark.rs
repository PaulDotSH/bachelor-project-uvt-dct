use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::{Serialize, Deserialize};
use bincode;
use redis::Client;
use tokio::runtime::Runtime;

// Structs that mimic the ones from the application for benchmarking
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Faculty {
    id: i32,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
enum Semester {
    First,
    Second,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Class {
    id: i32,
    name: String,
    descr: String,
    faculty: i32,
    semester: Semester,
    requirements: Option<String>,
    prof: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Filter {
    faculty: Option<i32>,
    semester: Option<Semester>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClassFaculty {
    classes: Vec<Class>,
    faculties: Vec<Faculty>,
}

fn create_sample_data() -> ClassFaculty {
    let classes = vec![
        Class {
            id: 1,
            name: "Test Class 1".to_string(),
            descr: "Description for test class 1".to_string(),
            faculty: 1,
            semester: Semester::First,
            requirements: Some("Some requirements".to_string()),
            prof: "Professor Name".to_string(),
        },
        Class {
            id: 2,
            name: "Test Class 2".to_string(),
            descr: "Description for test class 2".to_string(),
            faculty: 2,
            semester: Semester::Second,
            requirements: None,
            prof: "Another Professor".to_string(),
        },
    ];
    
    let faculties = vec![
        Faculty {
            id: 1,
            name: "Faculty of Science".to_string(),
        },
        Faculty {
            id: 2,
            name: "Faculty of Arts".to_string(),
        },
        Faculty {
            id: 3,
            name: "Faculty of Engineering".to_string(),
        },
    ];

    ClassFaculty { classes, faculties }
}

fn setup_redis() -> Result<Client, redis::RedisError> {
    redis::Client::open("redis://127.0.0.1/")
}

fn bench_redis_set(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    if let Ok(client) = setup_redis() {
        let mut con = match rt.block_on(async { 
            client.get_multiplexed_async_connection().await 
        }) {
            Ok(con) => con,
            Err(_) => {
                println!("Warning: Redis is not available. Skipping Redis benchmarks.");
                return;
            }
        };
        
        let data = create_sample_data();
        let encoded: Vec<u8> = bincode::serialize(&data).unwrap();
        
        c.bench_function("redis_set", |b| {
            b.iter(|| {
                let key = black_box("benchmark_key");
                let value = black_box(&encoded);
                let _: () = rt.block_on(async {
                    redis::cmd("SET")
                        .arg(key)
                        .arg(value)
                        .query_async(&mut con)
                        .await
                        .unwrap_or(())
                });
            })
        });
    } else {
        println!("Warning: Redis is not available. Skipping Redis benchmarks.");
    }
}

fn bench_redis_get(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    if let Ok(client) = setup_redis() {
        let mut con = match rt.block_on(async { 
            client.get_multiplexed_async_connection().await 
        }) {
            Ok(con) => con,
            Err(_) => {
                println!("Warning: Redis is not available. Skipping Redis benchmarks.");
                return;
            }
        };
        
        let data = create_sample_data();
        let encoded: Vec<u8> = bincode::serialize(&data).unwrap();
        let key = "benchmark_get_key";
        let _: () = rt.block_on(async {
            redis::cmd("SET")
                .arg(key)
                .arg(&encoded)
                .query_async(&mut con)
                .await
                .unwrap_or(())
        });
        
        c.bench_function("redis_get", |b| {
            b.iter(|| {
                let result: Vec<u8> = rt.block_on(async {
                    redis::cmd("GET")
                        .arg(black_box(key))
                        .query_async(&mut con)
                        .await
                        .unwrap_or_default()
                });
                if !result.is_empty() {
                    let _: ClassFaculty = bincode::deserialize(&result).unwrap_or_else(|_| ClassFaculty {
                        classes: Vec::new(),
                        faculties: Vec::new(),
                    });
                }
            })
        });
    } else {
        println!("Warning: Redis is not available. Skipping Redis benchmarks.");
    }
}

fn bench_redis_set_ex(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    if let Ok(client) = setup_redis() {
        let mut con = match rt.block_on(async { 
            client.get_multiplexed_async_connection().await 
        }) {
            Ok(con) => con,
            Err(_) => {
                println!("Warning: Redis is not available. Skipping Redis benchmarks.");
                return;
            }
        };
        
        let data = create_sample_data();
        let encoded: Vec<u8> = bincode::serialize(&data).unwrap();
        
        c.bench_function("redis_set_ex", |b| {
            b.iter(|| {
                let key = black_box("benchmark_key_ex");
                let value = black_box(&encoded);
                let expiry = black_box(600);
                let _: () = rt.block_on(async {
                    redis::cmd("SETEX")
                        .arg(key)
                        .arg(expiry)
                        .arg(value)
                        .query_async(&mut con)
                        .await
                        .unwrap_or(())
                });
            })
        });
    } else {
        println!("Warning: Redis is not available. Skipping Redis benchmarks.");
    }
}

criterion_group!(benches, bench_redis_set, bench_redis_get, bench_redis_set_ex);
criterion_main!(benches); 