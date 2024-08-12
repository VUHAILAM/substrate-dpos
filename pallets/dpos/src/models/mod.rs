pub mod candidate;
pub use candidate::*;
pub mod delegate;
pub use delegate::*;
pub type DispatchResultWithValue<T> = Result<T, sp_runtime::DispatchError>;