use crate::Algorithm;
use crate::Point;
use crate::errors::{NativeOsrmError, OsrmError};
use crate::r#match::{MatchRequest, MatchResponse};
use crate::native::Osrm;
use crate::nearest::{NearestRequest, NearestResponse};
use crate::route::{RouteRequest, RouteRequestBuilder, RouteResponse, SimpleRouteResponse};
use crate::table::{TableRequest, TableResponse};
use crate::trip::{TripRequest, TripResponse};

/// The engine for calling into osrm-backend natively.

#[derive(Clone, Debug)]
#[cfg_attr(doc, doc(cfg(feature = "native")))]
pub struct OsrmEngine {
    instance: Osrm,
}

impl OsrmEngine {
    /// Initialise the native engine.
    ///
    /// `base_map_path` should be a path with the .osrm file extension. Map extraction
    /// does not actually produce such a file, instead it creates numerous files with
    /// further extensions. For example, extracting `germany-latest.osm.pbf` will produce
    /// `germany-latest.osrm.fileIndex`, `germany-latest.osrm.geometry`, `germany-latest.osrm.cell_metrics`
    /// and many more. In which case `.../germany-latest.osrm` should be passed.
    ///
    /// The algorithm is determined by the extraction process. See the module level
    /// documentation for more information.
    pub fn new(base_map_path: &str, algorithm: Algorithm) -> Result<Self, OsrmError> {
        let osrm = Osrm::new(base_map_path, algorithm.as_str())
            .map_err(|e| OsrmError::Native(NativeOsrmError::Initialization(e)))?;
        Ok(OsrmEngine { instance: osrm })
    }

    /// Given a set of source and destination `Point`s or `Hint`s, determine the distances
    /// and/or durations to travel between all sources and destinations.
    ///
    /// See `TableRequest` for all possible options.
    ///
    /// ## Official documentation
    ///
    /// Computes the duration of the fastest route between all pairs of supplied coordinates.
    /// Returns durations or distances or both between the coordinate pairs. Note that the
    /// distances are not the shortest distance between two coordinates, but rather the
    /// distances of the fastest routes. Durations are in seconds and distances are in meters.
    pub fn table(&self, table_request: &TableRequest) -> Result<TableResponse, OsrmError> {
        let result = self
            .instance
            .table(table_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        serde_json::from_str::<TableResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))
    }

    /// Given an ordered set of `Point`s or `Hint`s, route through those points in the
    /// given order.
    ///
    /// See `RouteRequest` for all possible options.
    ///
    /// ## Official documentation
    ///
    /// Finds the fastest route between coordinates in the supplied order.
    pub fn route(&self, route_request: &RouteRequest) -> Result<RouteResponse, OsrmError> {
        let result = self
            .instance
            .route(route_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        serde_json::from_str::<RouteResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))
    }

    /// Given an _unordered_ set of `Point`s or `Hint`s, uses a greedy heuristic to
    /// approximately solve the travelling salesman problem. Returns the fastest route
    /// through those points in some order.
    ///
    /// See `TripRequest` for all possible options.
    ///
    /// ## Official documentation
    ///
    /// The trip plugin solves the Traveling Salesman Problem using a greedy
    /// heuristic (farthest-insertion algorithm) for 10 or more waypoints and
    /// uses brute force for less than 10 waypoints. The returned path does
    /// not have to be the fastest one. As TSP is NP-hard it only returns an
    /// approximation. Note that all input coordinates have to be connected
    /// for the trip service to work.
    pub fn trip(&self, trip_request: &TripRequest) -> Result<TripResponse, OsrmError> {
        let result = self
            .instance
            .trip(trip_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        serde_json::from_str::<TripResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))
    }

    /// A massively simplified interface for routing just between two points.
    ///
    /// Calls OsrmEngine::route with default options.
    pub fn simple_route(&self, from: Point, to: Point) -> Result<SimpleRouteResponse, OsrmError> {
        let points = [from, to];
        let request = RouteRequestBuilder::new(&points)
            .build()
            .expect("Route request for simple route is empty");

        let result = self
            .instance
            .route(&request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        let route_response = serde_json::from_str::<RouteResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))?;
        if route_response.routes.is_empty() {
            return Err(OsrmError::EmptyResponse(
                "No route was returned between those 2 points".to_owned(),
            ));
        }
        Ok(SimpleRouteResponse {
            code: route_response.code,
            distance: route_response
                .routes
                .first()
                .unwrap()
                .legs
                .iter()
                .map(|l| l.distance)
                .sum(),
            duration: route_response
                .routes
                .first()
                .unwrap()
                .legs
                .iter()
                .map(|l| l.duration)
                .sum(),
        })
    }

    /// Snap the given `Point` to the n closest nodes on the map. Returning the snapped
    /// coordinates and various metrics.
    ///
    /// `Hint`s returned from `nearest` may be passed to other services, allowing
    /// the call to skip the snapping process on subsequent calls.
    ///
    /// See `NearestRequest` for all possible options.
    ///
    /// ## Official documentation
    ///
    /// Snaps a coordinate to the street network and returns the nearest n matches.
    pub fn nearest(&self, nearest_request: &NearestRequest) -> Result<NearestResponse, OsrmError> {
        let result = self
            .instance
            .nearest(nearest_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        let nearest_response = serde_json::from_str::<NearestResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))?;
        Ok(nearest_response)
    }

    /// Given an ordered set of `Point`s or `Hint`s (and optionally
    /// timestamps), determine the likely route taken that could match
    /// those coordinates. Returns the route and confidence values.
    ///
    /// See `MatchRequest` for all possible options.
    ///
    /// ## Official documentation
    ///
    /// Map matching matches/snaps given GPS points to the road network
    /// in the most plausible way. Please note the request might result
    /// in multiple sub-traces. Large jumps in the timestamps (> 60s) or
    /// improbable transitions lead to trace splits if a complete matching
    /// could not be found. The algorithm might not be able to match all
    /// points. Outliers are removed if they can not be matched successfully.
    pub fn r#match(&self, match_request: &MatchRequest) -> Result<MatchResponse, OsrmError> {
        let result = self
            .instance
            .r#match(match_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        let response = serde_json::from_str::<MatchResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))?;

        Ok(response)
    }
}
