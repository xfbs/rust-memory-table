use crate::*;
use anyhow::anyhow;

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
    table.constraint_add("name_must_not_be_empty", |item: &Person| {
        if item.name.len() == 0 {
            Err(MyError::Fail)?
        } else {
            Ok(())
        }
    });

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
