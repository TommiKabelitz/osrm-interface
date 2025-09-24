use thiserror::Error;

#[derive(Error, Debug)]
pub enum NativeOsrmError {
    #[error("Failed to create OSRM instance")]
    Initialization,
    #[error("Invalid path parameter: {0}")]
    InvalidPath(String),
    #[error("Failed to parse OSRM response: {0}")]
    JsonParse(Box<dyn std::error::Error + Send + Sync>),
    #[error("Internal FFI error: {0}")]
    FfiError(String),
    #[error("Endpoint error: {0}")]
    EndpointError(String),
}

#[derive(Error, Debug)]
pub enum RemoteOsrmError {
    #[error("Failed to parse OSRM response: {0}")]
    JsonParse(Box<dyn std::error::Error + Send + Sync>),
    #[error("Endpoint error: {0}")]
    EndpointError(String),
}

#[derive(Error, Debug)]
pub enum OsrmError {
    #[error("Sources or destinations are invalid")]
    InvalidTableRequest,
    #[error("No points in request")]
    InvalidRouteRequest,
    #[error("No points in request")]
    InvalidTripRequest,
    #[error("Request produced an empty response: {0}")]
    EmptyResponse(String),
    #[error("Error from the native backend: {0}")]
    Native(NativeOsrmError),
    #[error("Error from the remote backend: {0}")]
    Remote(RemoteOsrmError),
}
