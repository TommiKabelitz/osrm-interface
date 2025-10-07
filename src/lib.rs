pub mod algorithm;
pub mod errors;
pub mod r#match;
pub mod nearest;
pub mod osrm_response_types;
pub mod point;
pub mod request_types;
pub mod route;
mod str_ops;
pub mod tables;
pub mod trip;

pub(crate) use str_ops::get_index_of_line_col;

#[cfg(feature = "native")]
pub mod native;

#[cfg(feature = "remote")]
pub mod remote;

pub mod mock;
