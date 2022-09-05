use deli::{Database, Model};
use serde::Deserialize;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Deserialize, Model)]
pub struct Employee {
    #[deli(auto_increment)]
    pub id: u32,
    pub name: String,
    #[deli(unique)]
    pub email: String,
    pub age: u8,
}

#[wasm_bindgen_test]
async fn test_db_creation_and_deletion() {
    let database = Database::new("sample".to_string(), 1)
        .await
        .expect("database");

    assert_eq!(database.database().name(), "sample");
    assert_eq!(database.database().version().unwrap(), 1);

    database.close();

    Database::delete("sample").await.expect("delete");
}

#[wasm_bindgen_test]
async fn test_model() {
    let mut builder = Database::builder("sample".to_string(), 1);
    builder.register_model::<Employee>();
    let database = builder.build().await.expect("database");

    let transaction = database
        .transaction()
        .writable()
        .with_model::<Employee>()
        .build()
        .expect("transaction");

    let id = Employee::with_transaction(&transaction)
        .add("Devashish", "devashishdxt@gmail.com", &32)
        .await
        .expect("employee add");

    assert_eq!(id, 1);

    transaction.commit().await.expect("transaction commit");

    database.close();
    Database::delete("sample").await.expect("delete");
}
