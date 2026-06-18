#![allow(clippy::module_inception)]

pub mod error;
pub mod result;

// ---------------------------------------------------------------------------------------------- //

pub(crate) use error::AppError;
pub(crate) use result::AppResult;
