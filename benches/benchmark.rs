use criterion::{black_box, criterion_group, criterion_main, Criterion};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use bincode;

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

fn bench_password_hash(c: &mut Criterion) {
    let password = "benchmark_password";
    
    c.bench_function("password_hash", |b| {
        b.iter(|| {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2.hash_password(black_box(password.as_bytes()), &salt).unwrap();
            black_box(password_hash.to_string())
        })
    });
}

fn bench_password_verify(c: &mut Criterion) {
    let password = "benchmark_password";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    let hash_string = password_hash.to_string();
    let parsed_hash = PasswordHash::new(&hash_string).unwrap();
    
    c.bench_function("password_verify", |b| {
        b.iter(|| {
            let verification_result = Argon2::default().verify_password(
                black_box(password.as_bytes()), 
                black_box(&parsed_hash)
            );
            black_box(verification_result)
        })
    });
}

fn bench_bincode(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bincode");

    let classes = vec![
        Class {
            id: 1,
            name: "Test Class 1".to_string(),
            descr: "Description for test class 1".to_string(),
            faculty: 2,
            semester: Semester::First,
            requirements: Some("Some requirements".to_string()),
            prof: "Professor Name".to_string(),
        },
        Class {
            id: 2,
            name: "Test Class 2".to_string(),
            descr: "Description for test class 2".to_string(),
            faculty: 3,
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

    let cf = ClassFaculty { classes: classes.clone(), faculties: faculties.clone() };
    
    group.bench_function("serialize", |b| {
        b.iter(|| {
            let encoded: Vec<u8> = bincode::serialize(black_box(&cf)).unwrap();
            black_box(encoded)
        })
    });
    
    let encoded: Vec<u8> = bincode::serialize(&cf).unwrap();
    
    group.bench_function("deserialize", |b| {
        b.iter(|| {
            let decoded: ClassFaculty = bincode::deserialize(black_box(&encoded[..])).unwrap();
            black_box(decoded)
        })
    });
    
    group.finish();
}

fn bench_filter_classes(c: &mut Criterion) {
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
        Class {
            id: 3,
            name: "Test Class 3".to_string(),
            descr: "Description for test class 3".to_string(),
            faculty: 1,
            semester: Semester::Second,
            requirements: None,
            prof: "Third Professor".to_string(),
        },
        Class {
            id: 4,
            name: "Test Class 4".to_string(),
            descr: "Description for test class 4".to_string(),
            faculty: 3,
            semester: Semester::First,
            requirements: Some("Other requirements".to_string()),
            prof: "Fourth Professor".to_string(),
        },
    ];
    
    c.bench_function("filter_by_faculty", |b| {
        b.iter(|| {
            let faculty_id = black_box(1);
            let filtered: Vec<&Class> = classes.iter()
                .filter(|class| class.faculty == faculty_id)
                .collect();
            black_box(filtered)
        })
    });
    
    c.bench_function("filter_by_semester", |b| {
        b.iter(|| {
            let semester = black_box(Semester::First);
            let filtered: Vec<&Class> = classes.iter()
                .filter(|class| class.semester == semester)
                .collect();
            black_box(filtered)
        })
    });
    
    c.bench_function("filter_by_faculty_and_semester", |b| {
        b.iter(|| {
            let faculty_id = black_box(1);
            let semester = black_box(Semester::Second);
            let filtered: Vec<&Class> = classes.iter()
                .filter(|class| class.faculty == faculty_id && class.semester == semester)
                .collect();
            black_box(filtered)
        })
    });
}

criterion_group!(
    benches, 
    bench_password_hash,
    bench_password_verify,
    bench_bincode,
    bench_filter_classes
);
criterion_main!(benches); 