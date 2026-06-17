use crate::error::app_error::AppError;

use std::result::Result as StdResult;

pub type AppResult<T> = StdResult<T, AppError>;
