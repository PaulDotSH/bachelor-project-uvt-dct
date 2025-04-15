use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};
use rand::distributions::{Alphanumeric, Distribution, Standard};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};

// Structs that mimic the ones from the application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Faculty {
    id: i32,
    name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum Semester {
    First,
    Second,
}

impl Hash for Semester {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Semester::First => 0.hash(state),
            Semester::Second => 1.hash(state),
        }
    }
}

impl Distribution<Semester> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Semester {
        match rng.gen_range(0..2) {
            0 => Semester::First,
            _ => Semester::Second,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Class {
    id: i32,
    name: String,
    descr: String,
    faculty: i32,
    semester: Semester,
    requirements: Option<String>,
    prof: String,
}

// Create a mock database with tables
struct MockDb {
    faculties: Vec<Faculty>,
    classes: Vec<Class>,
    faculty_index: HashMap<i32, usize>,
    classes_by_faculty: HashMap<i32, Vec<usize>>,
    classes_by_semester: HashMap<Semester, Vec<usize>>,
}

impl MockDb {
    fn new() -> Self {
        MockDb {
            faculties: Vec::new(),
            classes: Vec::new(),
            faculty_index: HashMap::new(),
            classes_by_faculty: HashMap::new(),
            classes_by_semester: HashMap::new(),
        }
    }

    fn insert_faculty(&mut self, faculty: Faculty) {
        let index = self.faculties.len();
        self.faculty_index.insert(faculty.id, index);
        self.faculties.push(faculty);
    }

    fn insert_class(&mut self, class: Class) {
        let index = self.classes.len();
        
        self.classes_by_faculty
            .entry(class.faculty)
            .or_insert_with(Vec::new)
            .push(index);
        
        self.classes_by_semester
            .entry(class.semester)
            .or_insert_with(Vec::new)
            .push(index);
        
        self.classes.push(class);
    }

    fn get_classes_by_faculty(&self, faculty_id: i32) -> Vec<&Class> {
        match self.classes_by_faculty.get(&faculty_id) {
            Some(indices) => indices.iter().map(|&i| &self.classes[i]).collect(),
            None => Vec::new(),
        }
    }

    fn get_classes_by_semester(&self, semester: Semester) -> Vec<&Class> {
        match self.classes_by_semester.get(&semester) {
            Some(indices) => indices.iter().map(|&i| &self.classes[i]).collect(),
            None => Vec::new(),
        }
    }

    fn get_classes_by_faculty_and_semester(&self, faculty_id: i32, semester: Semester) -> Vec<&Class> {
        let by_faculty = match self.classes_by_faculty.get(&faculty_id) {
            Some(indices) => indices,
            None => return Vec::new(),
        };
        
        let by_semester = match self.classes_by_semester.get(&semester) {
            Some(indices) => indices,
            None => return Vec::new(),
        };
        
        let mut result = Vec::new();
        for &i in by_faculty {
            if by_semester.contains(&i) {
                result.push(&self.classes[i]);
            }
        }
        
        result
    }

    fn get_all_classes(&self) -> Vec<&Class> {
        self.classes.iter().collect()
    }

    fn get_all_faculties(&self) -> Vec<&Faculty> {
        self.faculties.iter().collect()
    }
}

fn random_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn create_mock_db(faculty_count: usize, class_count: usize) -> MockDb {
    let mut db = MockDb::new();
    let mut rng = thread_rng();
    
    for i in 1..=faculty_count {
        let faculty = Faculty {
            id: i as i32,
            name: format!("Faculty of {}", random_string(10)),
        };
        db.insert_faculty(faculty);
    }
    
    for i in 1..=class_count {
        let faculty_id = rng.gen_range(1..=faculty_count) as i32;
        let class = Class {
            id: i as i32,
            name: format!("Class {}", random_string(15)),
            descr: random_string(50),
            faculty: faculty_id,
            semester: rand::random(),
            requirements: if rng.gen_bool(0.5) {
                Some(random_string(20))
            } else {
                None
            },
            prof: format!("Prof. {}", random_string(10)),
        };
        db.insert_class(class);
    }
    
    db
}

fn bench_get_all_classes(c: &mut Criterion) {
    let db = create_mock_db(5, 100);
    
    c.bench_function("db_get_all_classes", |b| {
        b.iter(|| {
            black_box(db.get_all_classes())
        })
    });
}

fn bench_get_classes_by_faculty(c: &mut Criterion) {
    let db = create_mock_db(5, 100);
    
    c.bench_function("db_get_classes_by_faculty", |b| {
        b.iter(|| {
            let faculty_id = black_box(2);
            black_box(db.get_classes_by_faculty(faculty_id))
        })
    });
}

fn bench_get_classes_by_semester(c: &mut Criterion) {
    let db = create_mock_db(5, 100);
    
    c.bench_function("db_get_classes_by_semester", |b| {
        b.iter(|| {
            let semester = black_box(Semester::First);
            black_box(db.get_classes_by_semester(semester))
        })
    });
}

fn bench_get_classes_by_faculty_and_semester(c: &mut Criterion) {
    let db = create_mock_db(5, 100);
    
    c.bench_function("db_get_classes_by_faculty_and_semester", |b| {
        b.iter(|| {
            let faculty_id = black_box(3);
            let semester = black_box(Semester::Second);
            black_box(db.get_classes_by_faculty_and_semester(faculty_id, semester))
        })
    });
}

fn bench_large_dataset(c: &mut Criterion) {
    let db = create_mock_db(20, 1000);
    
    let mut group = c.benchmark_group("large_dataset");
    
    group.bench_function("get_all_faculties", |b| {
        b.iter(|| {
            black_box(db.get_all_faculties())
        })
    });
    
    group.bench_function("get_all_classes", |b| {
        b.iter(|| {
            black_box(db.get_all_classes())
        })
    });
    
    group.bench_function("filtered_query", |b| {
        b.iter(|| {
            let faculty_id = black_box(10);
            let semester = black_box(Semester::First);
            black_box(db.get_classes_by_faculty_and_semester(faculty_id, semester))
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_get_all_classes, bench_get_classes_by_faculty,
                 bench_get_classes_by_semester, bench_get_classes_by_faculty_and_semester,
                 bench_large_dataset);
criterion_main!(benches); 