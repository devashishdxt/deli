/// Error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Indexed DB error
    #[error("indexed db error")]
    IndexedDbError(#[from] idb::Error),
    /// Full key range not allowed
    #[error("full key range not allowed")]
    FullKeyRangeNotAllowed,
    /// WASM serde error
    #[error("wasm serde error")]
    WasmSerdeError(#[from] serde_wasm_bindgen::Error),
}
