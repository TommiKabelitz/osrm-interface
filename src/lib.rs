//! osrm-interface provides a comprehensive interface to call into osrm-backend v6.0.0, the
//! open source routing machine. The crate provides the capability to call into the backend
//! natively if it is installed, alternatively through the web API.
//!
//! ## Engines
//!
//! The osrm engine is an instance which allows calling of the various services provided by osrm.
//! Three engines are provided. Their API's differ only in initialisation. Otherwise they are identical.
//!
//! To minimise compile times and binary sizes, the native and remote engines are disabled by default
//! and are gated by feature flags.
//!
//! - mock: For development convenience when the backend is otherwise unavailable.
//!   Returns the correct response types, but with fabricated data.
//! - remote (`feature="remote"`): Call using the web API. Works with `osrm-routed` running locally or
//!   remotely.
//! - native (`feature="native"`): Call by natively interfacing into an installed version of osrm-backend
//!   through a C++ wrapper. REQUIRES A LOCAL INSTALLATION OF `osrm-backend` AND LOCAL VERSIONS OF
//!   APPROPRIATELY EXTRACTED MAP DATA.
//!
//! For more information about initialising the engines and their requirements, see their module pages.
//!
//! ## Services
//!
//! The following services are presently supported
//!
//! The services use the [`Point`] struct which is initialised from a (latitude, longitude) pair to
//! store coordinates in a type safe mannner. Alternatively, exact points in the form of a `Hint`
//! (base64 encoded `String`), returned by nearest (and optionally the other services) which are
//! already snapped to the grid may be used.
//!
//! - nearest: Snap the given `Point` to the closest node on the map. Returning the snapped coordinates
//!   and other information.
//! - table: Given a set of source and destination `Point`s or `Hint`s, determine the distances and/or durations
//!   to travel between all sources and destinations.
//! - route: Given an ordered set of `Point`s or `Hint`s, route through those points in the given order.
//! - match: Given an ordered set of `Point`s or `Hint`s (and optionally timestamps), determine the likely
//!   route taken that could match those coordinates. Returns the route and confidence values.
//! - trip: Given an _unordered_ set of `Point`s or `Hint`s, uses a greedy heuristic to approximately solve
//!   the travelling salesman problem. Returns the fastest route through those points in some order.
//!
//! All service request are constructed using their corresponding request builders. These builders attempt to
//! verify whether the request will be rejected by OSRM before calling the service with transparent error messages.
//!
//! The builders also showcase the many options available for each service. See the builders (in the services)
//! for their available options.
//!
//! The tile service is not currently supported. I have no plans to add support, but a pull request is welcome
//! if someone would like to add it.
//!
//! ## Feature flags
//!
//! Feature flags are used aggressively to gate substantial portions of code from the crate to help with compile times.
//!
//! - `feature="native"`: Enable the native engine - will not compile without the ability to link a compiled version of
//!   osrm-backend.
//! - `feature="remote"`: Enable the remote engine for routing through the HTTP web API.
//! - `feature="serde"`: Add [`serde::Serialize`] and [`serde::Deserialize`] to all types. Response types require `Deserialize`
//!   when using the remote and native engines anyway, so the remote and native feature flags will enable this flag also.
//!
//! ## Example usage
//!
//! First initialise the appropriate engine. Then construct your requests with
//! the request builders.
//!
//! ```
//! // native engine requires map path and algorithm and returns a Result
//! // remote engine requires profile and an endpoint address
//! use osrm_interface::{Point, route::RouteRequestBuilder};
//! let engine = osrm_interface::mock::OsrmEngine::new();
//!
//! let points = [
//!     Point::new(48.040437, 10.316550).expect("Invalid point"),
//!     Point::new(49.006101, 9.052887).expect("Invalid point"),
//!     Point::new(48.942296, 10.510960).expect("Invalid point"),
//!     Point::new(51.248931, 7.594814).expect("Invalid point"),
//! ];
//! let route_request = RouteRequestBuilder::new(&points)
//!     .build()
//!     .expect("No points in request");
//! let response = engine
//!     .route(&route_request)
//!     .expect("Failed to route request");
//! ```
//!
//! ## A note on snapping
//!
//! The nearest service provides snapping of `Points` to nodes on the map. All
//! services will also snap `Points` to the map. To avoid snapping, pass `Hints`
//! to the various services which allows the service to skip the snapping which
//! has already been done. `Hints` can be returned by all services and are returned
//! by default.
//!
//! In addition, be aware that the nearest node that can be snapped to is in no
//! way limited by distance. If the backend is running on the Germany map for
//! example, and the user requests a point with a lat/long in the USA, then the
//! snapping will return the closest node in the German map (likely on the
//! south-western side). This is also true when points near country borders are
//! snapped using map data which does not contain both countries.
//!
//! ## Serialisation/Deserialisation
//!
//! Presently, both the native and remote engines generate/request json to serialise the responses. osrm-backend supports
//! a more efficient flatbuffer format which is exposed through their include headers. I plan to add support for this
//! format in the future at which point the two forms will both be supported, with the flatbuffer format providing better
//! format, and the json format providing human readability.
//!
//! ## General OSRM documentation
//!
//! The main [osrm-backend documentation](<https://github.com/Project-OSRM/osrm-backend/wiki>) is located on the osrm-backend
//! Github. The HTTP API of `osrm-routed` is a good source of information.
#![cfg_attr(all(doc), feature(doc_cfg))]

pub mod services;
pub use services::{r#match, nearest, route, table, trip};

pub mod errors;
pub mod osrm_response_types;
pub mod request_types;
mod str_ops;

pub(crate) use str_ops::get_index_of_line_col;

#[cfg(feature = "native")]
pub mod native;

#[cfg(feature = "remote")]
pub mod remote;

pub mod mock;

/// The algorithm used in the pre-processing pipeline to
/// generate the .osrm map files.
///
/// MLD: Multi-level Dijkstra
///
/// CH: Contraction Hierarchies
///
/// Implements [`serde::Deserialize`] and
/// [`serde::Serialize`] if `feature="serde"` is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    /// Multi-level Dijkstra
    MLD,
    /// Contraction Hierarchies
    CH,
}

impl Algorithm {
    /// Returns the variant only, as a capitalised static str.
    ///
    /// eg. `"MLD"` or `"CH"`
    pub fn as_str(&self) -> &str {
        match self {
            Algorithm::MLD => "MLD",
            Algorithm::CH => "CH",
        }
    }
}

/// A (latitude, longitude pair). The basic coordinate type to pass to OSRM.
///
/// Constructing with `new` will check -90 <= latitude <= 90 and
/// -180 <= longitude <= 180, returning an Option<>.
///
/// [`new_unchecked`](Self::new_unchecked) is also provided.
///
/// Implements [`serde::Deserialize`] and
/// [`serde::Serialize`] if `feature="serde"` is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    latitude: f64,
    longitude: f64,
}

impl Point {
    /// Checks -90 <= latitude <= 90 and -180 <= longitude <= 180.
    ///
    /// Returns `None` if that is not satisfied. Also see [`new_unchecked`](Self::new_unchecked).
    pub fn new(latitude: f64, longitude: f64) -> Option<Self> {
        // Range contains produces the same assembly as chained <= and >= with optimisation
        if !((-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude)) {
            return None;
        }
        Some(Self {
            latitude,
            longitude,
        })
    }

    /// Init without checking latitude and longitude.
    ///
    /// OSRM will reject points
    /// outside the legal latitude and longitude ranges when a request is made to
    /// a service.
    pub fn new_unchecked(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }
}
