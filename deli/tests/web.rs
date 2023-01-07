use deli::{Database, Direction, Error, Model, Transaction};
use serde::Deserialize;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Deserialize, Model)]
#[allow(dead_code)]
struct Employee {
    #[deli(auto_increment)]
    id: u32,
    name: String,
    #[deli(unique)]
    email: String,
    #[deli(index)]
    age: u8,
}

async fn create_database() -> Result<Database, Error> {
    let _ = Database::delete("test_db").await;

    Database::builder("test_db".to_string())
        .version(1)
        .register_model::<Employee>()
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

    let id = store.add("Alice", "alice@example.com", &25).await.unwrap();

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

    let id = store.add("Alice", "alice@example.com", &25).await.unwrap();

    let update_id = store
        .update(&id, "Bob", "bob@example.com", &30)
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

    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();

    let count = store.count(..).await.unwrap();
    assert_eq!(count, 2);

    let count = store.count(&0..&id2).await.unwrap();
    assert_eq!(count, 1);

    let count = store.count(&id2..=&id2).await.unwrap();
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

    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();

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

    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();

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
async fn test_scan() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();

    let forward_employees = store
        .scan(.., Some(Direction::Next), None, None)
        .await
        .unwrap();

    let backward_employees = store
        .scan(.., Some(Direction::Prev), None, None)
        .await
        .unwrap();

    assert_eq!(forward_employees.len(), 2);
    assert_eq!(backward_employees.len(), 2);

    assert_eq!(forward_employees[0].id, backward_employees[1].id);
    assert_eq!(forward_employees[0].name, backward_employees[1].name);
    assert_eq!(forward_employees[0].email, backward_employees[1].email);
    assert_eq!(forward_employees[0].age, backward_employees[1].age);

    assert_eq!(forward_employees[1].id, backward_employees[0].id);
    assert_eq!(forward_employees[1].name, backward_employees[0].name);
    assert_eq!(forward_employees[1].email, backward_employees[0].email);
    assert_eq!(forward_employees[1].age, backward_employees[0].age);

    let forward_employees = store
        .scan(.., Some(Direction::Next), Some(1), None)
        .await
        .unwrap();

    let backward_employees = store
        .scan(.., Some(Direction::Prev), Some(1), None)
        .await
        .unwrap();

    assert_eq!(forward_employees.len(), 1);
    assert_eq!(backward_employees.len(), 1);

    if forward_employees[0].id == id1 {
        assert_eq!(forward_employees[0].id, id1);
        assert_eq!(forward_employees[0].name, "Alice");
        assert_eq!(forward_employees[0].email, "alice@example.com");
        assert_eq!(forward_employees[0].age, 25);

        assert_eq!(backward_employees[0].id, id2);
        assert_eq!(backward_employees[0].name, "Bob");
        assert_eq!(backward_employees[0].email, "bob@example.com");
        assert_eq!(backward_employees[0].age, 30);
    } else {
        assert_eq!(backward_employees[0].id, id1);
        assert_eq!(backward_employees[0].name, "Alice");
        assert_eq!(backward_employees[0].email, "alice@example.com");
        assert_eq!(backward_employees[0].age, 25);

        assert_eq!(forward_employees[0].id, id2);
        assert_eq!(forward_employees[0].name, "Bob");
        assert_eq!(forward_employees[0].email, "bob@example.com");
        assert_eq!(forward_employees[0].age, 30);
    }

    let forward_employees = store
        .scan(.., Some(Direction::Next), Some(1), Some(1))
        .await
        .unwrap();

    let backward_employees = store
        .scan(.., Some(Direction::Prev), Some(1), None)
        .await
        .unwrap();

    assert_eq!(forward_employees.len(), 1);
    assert_eq!(backward_employees.len(), 1);

    assert_eq!(forward_employees[0].id, backward_employees[0].id);
    assert_eq!(forward_employees[0].name, backward_employees[0].name);
    assert_eq!(forward_employees[0].email, backward_employees[0].email);
    assert_eq!(forward_employees[0].age, backward_employees[0].age);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_scan_keys() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();

    let forward_employees = store
        .scan_keys(.., Some(Direction::Next), None, None)
        .await
        .unwrap();

    let backward_employees = store
        .scan_keys(.., Some(Direction::Prev), None, None)
        .await
        .unwrap();

    assert_eq!(forward_employees.len(), 2);
    assert_eq!(backward_employees.len(), 2);

    assert_eq!(forward_employees[0], backward_employees[1]);
    assert_eq!(forward_employees[1], backward_employees[0]);

    let forward_employees = store
        .scan_keys(.., Some(Direction::Next), Some(1), None)
        .await
        .unwrap();

    let backward_employees = store
        .scan_keys(.., Some(Direction::Prev), Some(1), None)
        .await
        .unwrap();

    assert_eq!(forward_employees.len(), 1);
    assert_eq!(backward_employees.len(), 1);

    if forward_employees[0] == id1 {
        assert_eq!(forward_employees[0], id1);
        assert_eq!(backward_employees[0], id2);
    } else {
        assert_eq!(backward_employees[0], id1);
        assert_eq!(forward_employees[0], id2);
    }

    let forward_employees = store
        .scan_keys(.., Some(Direction::Next), Some(1), Some(1))
        .await
        .unwrap();

    let backward_employees = store
        .scan_keys(.., Some(Direction::Prev), Some(1), None)
        .await
        .unwrap();

    assert_eq!(forward_employees.len(), 1);
    assert_eq!(backward_employees.len(), 1);

    assert_eq!(forward_employees[0], backward_employees[0]);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_delete() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();

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

    store.add("Alice", "alice@example.com", &25).await.unwrap();

    let err = store.add("Bob", "alice@example.com", &30).await;
    assert!(err.is_err(), "{}", err.unwrap());
}

#[wasm_bindgen_test]
async fn test_count_by_index() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    store.add("Alice", "alice@example.com", &25).await.unwrap();
    store.add("Bob", "bob@example.com", &30).await.unwrap();
    store
        .add("Charlie", "charlie@example.com", &35)
        .await
        .unwrap();
    store.add("Dave", "dave@example.com", &40).await.unwrap();

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

    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();

    let employee1 = store
        .by_email()
        .unwrap()
        .get("alice@example.com")
        .await
        .unwrap();

    assert!(employee1.is_some());
    let employee1 = employee1.unwrap();

    assert_eq!(employee1.id, id1);

    let employee2 = store
        .by_email()
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

    let id = store.add("Alice", "alice@example.com", &25).await.unwrap();

    let key = store
        .by_email()
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

    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();
    let id3 = store
        .add("Charlie", "charlie@example.com", &35)
        .await
        .unwrap();
    let id4 = store.add("Dave", "dave@example.com", &40).await.unwrap();

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

    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();
    let id3 = store
        .add("Charlie", "charlie@example.com", &35)
        .await
        .unwrap();
    let id4 = store.add("Dave", "dave@example.com", &40).await.unwrap();

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
        .get_all_keys(&35..&40, None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 1);
    assert_eq!(employees[0], id3);

    let employees = store
        .by_age()
        .unwrap()
        .get_all_keys(&35..=&40, Some(1))
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

#[wasm_bindgen_test]
async fn test_scan_by_index() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id4 = store.add("Dave", "dave@example.com", &40).await.unwrap();
    let id3 = store
        .add("Charlie", "charlie@example.com", &35)
        .await
        .unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();
    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();

    let employees = store
        .by_age()
        .unwrap()
        .scan(..=&30, Some(Direction::Next), None, None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 2);

    assert_eq!(employees[0].id, id1);
    assert_eq!(employees[1].id, id2);

    let employees = store
        .by_age()
        .unwrap()
        .scan(&31.., Some(Direction::Prev), None, None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 2);

    assert_eq!(employees[0].id, id4);
    assert_eq!(employees[1].id, id3);

    let employees = store
        .by_age()
        .unwrap()
        .scan(..=&30, Some(Direction::Next), Some(1), None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 1);

    assert_eq!(employees[0].id, id1);

    let employees = store
        .by_age()
        .unwrap()
        .scan(..=&30, Some(Direction::Next), None, Some(1))
        .await
        .unwrap();

    assert_eq!(employees.len(), 1);

    assert_eq!(employees[0].id, id2);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}

#[wasm_bindgen_test]
async fn test_scan_keys_by_index() {
    let database = create_database().await.unwrap();
    let transaction = begin_write_transaction(&database).unwrap();
    let store = Employee::with_transaction(&transaction).unwrap();

    let id4 = store.add("Dave", "dave@example.com", &40).await.unwrap();
    let id3 = store
        .add("Charlie", "charlie@example.com", &35)
        .await
        .unwrap();
    let id2 = store.add("Bob", "bob@example.com", &30).await.unwrap();
    let id1 = store.add("Alice", "alice@example.com", &25).await.unwrap();

    let employees = store
        .by_age()
        .unwrap()
        .scan_keys(..=&30, Some(Direction::Next), None, None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 2);

    assert_eq!(employees[0], id1);
    assert_eq!(employees[1], id2);

    let employees = store
        .by_age()
        .unwrap()
        .scan_keys(&31.., Some(Direction::Prev), None, None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 2);

    assert_eq!(employees[0], id4);
    assert_eq!(employees[1], id3);

    let employees = store
        .by_age()
        .unwrap()
        .scan_keys(..=&30, Some(Direction::Next), Some(1), None)
        .await
        .unwrap();

    assert_eq!(employees.len(), 1);

    assert_eq!(employees[0], id1);

    let employees = store
        .by_age()
        .unwrap()
        .scan_keys(..=&30, Some(Direction::Next), None, Some(1))
        .await
        .unwrap();

    assert_eq!(employees.len(), 1);

    assert_eq!(employees[0], id2);

    transaction.done().await.expect("transaction done");

    close_and_delete_database(database).await.unwrap();
}
