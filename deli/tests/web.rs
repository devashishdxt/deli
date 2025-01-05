use deli::{Database, Error, Model, Transaction};
use serde::{Deserialize, Serialize};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Serialize, Deserialize, Model)]
struct Employee {
    #[deli(auto_increment)]
    id: u32,
    name: String,
    #[deli(unique)]
    email: String,
    #[deli(index)]
    age: u32,
}

async fn create_database() -> Result<Database, Error> {
    let _ = Database::delete("test_db").await;

    Database::builder("test_db")
        .version(1)
        .add_model::<Employee>()
        .build()
        .await
}

fn begin_read_transaction(database: &Database) -> Result<Transaction, Error> {
    database.transaction().with_model::<Employee>().build()
}

fn begin_write_transaction(database: &Database) -> Result<Transaction, Error> {
    database
        .transaction()
        .writable()
        .with_model::<Employee>()
        .build()
}

async fn close_and_delete_database(database: Database) -> Result<(), Error> {
    database.close();
    Database::delete("test_db").await
}

#[wasm_bindgen_test]
async fn test_db_creation_and_deletion() {
    let database = create_database().await;
    assert!(database.is_ok(), "{:?}", database.unwrap_err());
    let database = database.unwrap();

    assert_eq!(database.name(), "test_db");
    assert_eq!(database.version().unwrap(), 1);

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_value_add() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id = store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();

    transaction.commit().await.unwrap();

    let transaction = begin_read_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let employee = store.get(&id).await.unwrap();
    assert!(employee.is_some());
    let employee = employee.unwrap();

    assert_eq!(employee.name, "Alice");
    assert_eq!(employee.email, "alice@example.com");
    assert_eq!(employee.age, 25);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_value_update() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id = store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();

    let update_id = store
        .update(&Employee {
            id,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        })
        .await
        .unwrap();

    assert_eq!(id, update_id);

    transaction.commit().await.unwrap();

    let transaction = begin_read_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let employee = store.get(&id).await.unwrap();
    assert!(employee.is_some());
    let employee = employee.unwrap();

    assert_eq!(employee.name, "Bob");
    assert_eq!(employee.email, "bob@example.com");
    assert_eq!(employee.age, 30);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_count() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let count = store.count(..).await.unwrap();
    assert_eq!(count, 0);

    let id1 = store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();
    let id2 = store
        .add(&AddEmployee {
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        })
        .await
        .unwrap();

    let count = store.count(..).await.unwrap();
    assert_eq!(count, 2);

    let count = store.count(..&id2).await.unwrap();
    assert_eq!(count, 1);

    let count = store.count(&id2).await.unwrap();
    assert_eq!(count, 1);

    let count = store.count(&id1..=&id2).await.unwrap();
    assert_eq!(count, 2);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_get_all() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let count = store.count(..).await.unwrap();
    assert_eq!(count, 0);

    let id1 = store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();
    let id2 = store
        .add(&AddEmployee {
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        })
        .await
        .unwrap();

    let all_employees = store.get_all(.., None).await.unwrap();

    assert_eq!(all_employees.len(), 2);

    if all_employees[0].id == id1 {
        assert_eq!(all_employees[0].id, id1);
        assert_eq!(all_employees[0].name, "Alice");
        assert_eq!(all_employees[0].email, "alice@example.com");
        assert_eq!(all_employees[0].age, 25);

        assert_eq!(all_employees[1].id, id2);
        assert_eq!(all_employees[1].name, "Bob");
        assert_eq!(all_employees[1].email, "bob@example.com");
        assert_eq!(all_employees[1].age, 30);
    } else {
        assert_eq!(all_employees[1].id, id1);
        assert_eq!(all_employees[1].name, "Alice");
        assert_eq!(all_employees[1].email, "alice@example.com");
        assert_eq!(all_employees[1].age, 25);

        assert_eq!(all_employees[0].id, id2);
        assert_eq!(all_employees[0].name, "Bob");
        assert_eq!(all_employees[0].email, "bob@example.com");
        assert_eq!(all_employees[0].age, 30);
    }

    let all_employees = store.get_all(&id2.., None).await.unwrap();

    assert_eq!(all_employees.len(), 1);
    assert_eq!(all_employees[0].id, id2);
    assert_eq!(all_employees[0].name, "Bob");
    assert_eq!(all_employees[0].email, "bob@example.com");
    assert_eq!(all_employees[0].age, 30);

    let all_employees = store.get_all(.., Some(1)).await.unwrap();

    assert_eq!(all_employees.len(), 1);

    if all_employees[0].id == id1 {
        assert_eq!(all_employees[0].id, id1);
        assert_eq!(all_employees[0].name, "Alice");
        assert_eq!(all_employees[0].email, "alice@example.com");
        assert_eq!(all_employees[0].age, 25);
    } else {
        assert_eq!(all_employees[0].id, id2);
        assert_eq!(all_employees[0].name, "Bob");
        assert_eq!(all_employees[0].email, "bob@example.com");
        assert_eq!(all_employees[0].age, 30);
    }

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_get_all_keys() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id1 = store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();
    let id2 = store
        .add(&AddEmployee {
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        })
        .await
        .unwrap();

    let all_employees = store.get_all_keys(.., None).await.unwrap();

    assert_eq!(all_employees.len(), 2);

    if all_employees[0] == id1 {
        assert_eq!(all_employees[0], id1);
        assert_eq!(all_employees[1], id2);
    } else {
        assert_eq!(all_employees[1], id1);
        assert_eq!(all_employees[0], id2);
    }

    let all_employees = store.get_all_keys(&id2.., None).await.unwrap();

    assert_eq!(all_employees.len(), 1);
    assert_eq!(all_employees[0], id2);

    let all_employees = store.get_all_keys(.., Some(1)).await.unwrap();

    assert_eq!(all_employees.len(), 1);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_delete() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id1 = store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();
    let id2 = store
        .add(&AddEmployee {
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        })
        .await
        .unwrap();

    let employee1 = store.get(&id1).await.unwrap();
    let employee2 = store.get(&id2).await.unwrap();

    assert!(employee1.is_some());
    assert!(employee2.is_some());

    store.delete(&id1).await.unwrap();

    let employee1 = store.get(&id1).await.unwrap();
    let employee2 = store.get(&id2).await.unwrap();

    assert!(employee1.is_none());
    assert!(employee2.is_some());

    store.delete(&id2..).await.unwrap();

    let employee1 = store.get(&id1).await.unwrap();
    let employee2 = store.get(&id2).await.unwrap();

    assert!(employee1.is_none());
    assert!(employee2.is_none());

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_unique_index() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();

    let err = store
        .add(&AddEmployee {
            name: "Bob".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        })
        .await;
    assert!(err.is_err(), "{:?}", err.unwrap());

    let transaction_result = transaction.done().await;
    assert!(
        transaction_result.is_err(),
        "{:?}",
        transaction_result.unwrap()
    );

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_count_by_index() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();
    store
        .add(&AddEmployee {
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        })
        .await
        .unwrap();

    store
        .add(&AddEmployee {
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            age: 35,
        })
        .await
        .unwrap();
    store
        .add(&AddEmployee {
            name: "Dave".to_string(),
            email: "dave@example.com".to_string(),
            age: 40,
        })
        .await
        .unwrap();

    let count = store.by_age().unwrap().count(..).await.unwrap();
    assert_eq!(count, 4);

    let count = store.by_age().unwrap().count(..=&30).await.unwrap();
    assert_eq!(count, 2);

    let count = store.by_age().unwrap().count(&31..).await.unwrap();
    assert_eq!(count, 2);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_get_by_index() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id1 = store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();
    let id2 = store
        .add(&AddEmployee {
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        })
        .await
        .unwrap();

    let employee1 = store
        .by_email_unique()
        .unwrap()
        .get("alice@example.com")
        .await
        .unwrap();

    assert!(employee1.is_some());
    let employee1 = employee1.unwrap();

    assert_eq!(employee1.id, id1);

    let employee2 = store
        .by_email_unique()
        .unwrap()
        .get("bob@example.com")
        .await
        .unwrap();

    assert!(employee2.is_some());
    let employee2 = employee2.unwrap();

    assert_eq!(employee2.id, id2);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_get_key_by_index() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id = store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();

    let key = store
        .by_email_unique()
        .unwrap()
        .get_key("alice@example.com")
        .await
        .unwrap();

    assert!(key.is_some());
    let key = key.unwrap();

    assert_eq!(key, id);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_get_all_by_index() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id1 = store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();
    let id2 = store
        .add(&AddEmployee {
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        })
        .await
        .unwrap();

    let id3 = store
        .add(&AddEmployee {
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            age: 35,
        })
        .await
        .unwrap();
    let id4 = store
        .add(&AddEmployee {
            name: "Dave".to_string(),
            email: "dave@example.com".to_string(),
            age: 40,
        })
        .await
        .unwrap();

    let employees = store.by_age().unwrap().get_all(..=&30, None).await.unwrap();

    assert_eq!(employees.len(), 2);

    if employees[0].id == id1 {
        assert_eq!(employees[1].id, id2);
    } else {
        assert_eq!(employees[0].id, id2);
        assert_eq!(employees[1].id, id1);
    }

    let employees = store
        .by_age()
        .unwrap()
        .get_all(&35..&40, None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 1);
    assert_eq!(employees[0].id, id3);

    let employees = store
        .by_age()
        .unwrap()
        .get_all(&35..=&40, Some(1))
        .await
        .unwrap();

    assert_eq!(employees.len(), 1);

    let employees = store.by_age().unwrap().get_all(&40.., None).await.unwrap();

    assert_eq!(employees.len(), 1);
    assert_eq!(employees[0].id, id4);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_get_all_keys_by_index() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id1 = store
        .add(&AddEmployee {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        })
        .await
        .unwrap();
    let id2 = store
        .add(&AddEmployee {
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        })
        .await
        .unwrap();

    let id3 = store
        .add(&AddEmployee {
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            age: 35,
        })
        .await
        .unwrap();
    let id4 = store
        .add(&AddEmployee {
            name: "Dave".to_string(),
            email: "dave@example.com".to_string(),
            age: 40,
        })
        .await
        .unwrap();

    let employees = store
        .by_age()
        .unwrap()
        .get_all_keys(..=&30, None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 2);

    if employees[0] == id1 {
        assert_eq!(employees[1], id2);
    } else {
        assert_eq!(employees[0], id2);
        assert_eq!(employees[1], id1);
    }

    let employees = store
        .by_age()
        .unwrap()
        .get_all_keys(&30..&40, None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 2);
    assert_eq!(employees[0], id2);
    assert_eq!(employees[1], id3);

    let employees = store
        .by_age()
        .unwrap()
        .get_all_keys(&30..=&40, Some(1))
        .await
        .unwrap();

    assert_eq!(employees.len(), 1);

    let employees = store
        .by_age()
        .unwrap()
        .get_all_keys(&40.., None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 1);
    assert_eq!(employees[0], id4);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}
