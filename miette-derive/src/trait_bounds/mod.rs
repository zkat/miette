#[cfg(not(feature = "perfect-derive"))]
mod mock_store;
#[cfg(not(feature = "perfect-derive"))]
pub use mock_store::TypeParamBoundStore;

#[cfg(feature = "perfect-derive")]
mod store;
#[cfg(feature = "perfect-derive")]
pub use store::TypeParamBoundStore;
