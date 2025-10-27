//! The top level error types returned from the services.
//! [`NativeOsrmError`], [`RemoteOsrmError`] and [`OsrmError`]
//! which includes the former pair.

use thiserror::Error;

/// Errors specifically from calling a service using the native
/// OSRM engine.
#[derive(Error, Debug)]
pub enum NativeOsrmError {
    /// Failed to create OSRM instance.
    #[error("Failed to create OSRM instance: {0}")]
    Initialization(String),
    /// Invalid path to map data.
    #[error("Invalid path parameter: {0}")]
    InvalidPath(String),
    /// Failed to parse OSRM response.
    #[error("Failed to parse OSRM response: {0}")]
    JsonParse(Box<dyn std::error::Error + Send + Sync>),
    /// Other FFI error.
    #[error("Internal FFI error: {0}")]
    FfiError(String),
}

/// Errors specifically from calling a service using the remote
/// OSRM engine.
#[derive(Error, Debug)]
pub enum RemoteOsrmError {
    // Failed to parse OSRM response.
    #[error("Failed to parse OSRM response: {0}")]
    JsonParse(Box<dyn std::error::Error + Send + Sync>),
    /// Other error from the request.
    #[error("Endpoint error: {0}")]
    EndpointError(String),
}

/// A union type for OSRM errors when calling a service.
///
/// Not to be confused with the various request errors which
/// are returned when attempting to _build_ a request.
#[derive(Error, Debug)]
pub enum OsrmError {
    /// Request produced an empty response.
    #[error("Request produced an empty response: {0}")]
    EmptyResponse(String),
    #[error("Error from the native backend: {0}")]
    Native(NativeOsrmError),
    #[error("Error from the remote backend: {0}")]
    Remote(RemoteOsrmError),
}
