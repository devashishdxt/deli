use thiserror::Error;

/// Error type for [`deli`](crate) crate
#[derive(Debug, Error)]
pub enum Error {
    /// Indexed DB error
    #[error("indexed db error")]
    IndexedDbError(#[from] idb::Error),

    /// Serde WASM error
    #[error("serde wasm error")]
    SerdeWasmError(#[from] serde_wasm_bindgen::Error),
}
