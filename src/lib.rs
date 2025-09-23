pub mod algorithm;
pub mod errors;
pub mod point;
pub mod request_types;
pub mod route;
pub mod tables;
pub mod trip;
pub mod waypoints;

#[cfg(osrm_native)]
mod native;
#[cfg(osrm_native)]
pub use native::OsrmEngine;

#[cfg(osrm_remote)]
mod remote;
#[cfg(osrm_remote)]
pub use remote::OsrmEngine;

#[cfg(osrm_mock)]
mod mock;
#[cfg(osrm_mock)]
pub use mock::OsrmEngine;
