use crate::*;
use anyhow::anyhow;
use rand::distributions::{Alphanumeric, DistString};
use rand::*;

#[derive(Debug, Clone)]
struct Person {
    id: u64,
    name: String,
    age: u16,
}

impl Identity for Person {
    type PrimaryKey = u64;
    fn primary_key(&self) -> Self::PrimaryKey {
        self.id
    }
}

#[derive(thiserror::Error, Debug)]
enum MyError {
    #[error("Something failed")]
    Fail,
}

#[test]
fn can_count_entries() {
    let mut table = Table::new();
    assert_eq!(table.len(), 0);
    table
        .insert(Person {
            id: 0,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();
    assert_eq!(table.len(), 1);
    table
        .insert(Person {
            id: 1,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();
    assert_eq!(table.len(), 2);
}

#[test]
fn can_clear_entries() {
    let mut table = Table::new();
    table
        .insert(Person {
            id: 0,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();
    table
        .insert(Person {
            id: 1,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();
    assert_eq!(table.len(), 2);
    table.clear();
    assert_eq!(table.len(), 0);
    assert!(table.lookup(&0).is_none());
    assert!(table.lookup(&1).is_none());
}

#[test]
fn can_create_person_table() {
    let mut table = Table::new();
    table.insert(Person {
        id: 0,
        name: "Mike".into(),
        age: 32,
    });
}

#[test]
fn can_set_primary_key_in_pre_insert_hook() {
    let mut table = Table::new();
    table.pre_insert_hook_add("primary_key", |table: &mut Table<Person>, item| {
        item.id = table.len() as u64;
    });
    for _ in 0..64 {
        let key = table
            .insert(Person {
                id: 0,
                name: "Mike".into(),
                age: 32,
            })
            .unwrap();
        assert_eq!(1 + key as usize, table.len());
    }
}

#[test]
fn cannot_insert_duplicate_primary_key() {
    let mut table = Table::new();
    let key = table
        .insert(Person {
            id: 0,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();
    assert_eq!(key, 0);

    // inserting same data should fail
    let result = table.insert(Person {
        id: 0,
        name: "Mike".into(),
        age: 32,
    });
    assert!(matches!(result, Err(TableError::Exists(0))));
}

#[test]
fn cannot_insert_failing_constraint() {
    let mut table = Table::new();
    table
        .constraint_add("name_must_not_be_empty", |item: &Person| {
            if item.name.len() == 0 {
                Err(MyError::Fail)?
            } else {
                Ok(())
            }
        })
        .unwrap();

    let key = table
        .insert(Person {
            id: 0,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();
    assert_eq!(key, 0);

    // inserting same data should fail
    let result = table.insert(Person {
        id: 1,
        name: "".into(),
        age: 32,
    });
    assert!(result.is_err());
}

#[test]
fn cannot_insert_failing_constraint_after() {
    let mut table = Table::new();
    table
        .insert(Person {
            id: 0,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();
    table
        .insert(Person {
            id: 1,
            name: "".into(),
            age: 32,
        })
        .unwrap();

    let result = table.constraint_add("name_must_not_be_empty", |item: &Person| {
        if item.name.len() == 0 {
            Err(MyError::Fail)?
        } else {
            Ok(())
        }
    });
    assert!(result.is_err());
}

#[test]
fn cannot_insert_duplicate_unique_index() {
    let mut table = Table::new();
    table
        .index_add(
            "name",
            UniqueBTreeIndex::new(|item: &Person| item.name.clone()),
        )
        .unwrap();
    let key = table
        .insert(Person {
            id: 0,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();

    // inserting same data should fail
    let result = table.insert(Person {
        id: 1,
        name: "Mike".into(),
        age: 32,
    });
    assert!(result.is_err());
}

#[test]
fn can_insert_multiple_unique_index() {
    let mut table = Table::new();
    table
        .index_add(
            "name",
            UniqueBTreeIndex::new(|item: &Person| item.name.clone()),
        )
        .unwrap();
    let key = table
        .insert(Person {
            id: 0,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();

    // inserting same data should fail
    let result = table
        .insert(Person {
            id: 1,
            name: "John".into(),
            age: 32,
        })
        .unwrap();
}

#[test]
fn can_insert_duplicate_index() {
    let mut table = Table::new();
    table
        .index_add("name", BTreeIndex::new(|item: &Person| item.name.clone()))
        .unwrap();
    let key = table
        .insert(Person {
            id: 0,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();

    // inserting same data should fail
    let result = table
        .insert(Person {
            id: 1,
            name: "Mike".into(),
            age: 32,
        })
        .unwrap();
}

#[test]
fn can_insert_one_many_rows() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(23420292352);
    let amount = 100_000;
    let mut table = Table::new();

    // auto-increment primary key
    table.pre_insert_hook_add("primary_key", |table: &mut Table<Person>, item| {
        item.id = table.len() as u64;
    });

    // constraint to make sure age is valid
    table
        .constraint_add("age", |item: &Person| {
            if item.age > 100 {
                Err(MyError::Fail)?
            } else {
                Ok(())
            }
        })
        .unwrap();

    // add unique name index
    table
        .index_add(
            "name",
            UniqueBTreeIndex::new(|item: &Person| item.name.clone()),
        )
        .unwrap();

    // add age index
    table
        .index_add("age", BTreeIndex::new(|item: &Person| item.age))
        .unwrap();

    for i in 0..amount {
        table
            .insert(Person {
                id: 0,
                name: Alphanumeric.sample_string(&mut rng, 16),
                age: rng.gen_range(0..100),
            })
            .unwrap();
    }

    assert_eq!(table.len(), amount);
}
