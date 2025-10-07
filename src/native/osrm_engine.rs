use crate::algorithm;
use crate::errors::{NativeOsrmError, OsrmError};
use crate::r#match::{MatchGapsBehaviour, MatchRequest, MatchResponse};
use crate::native::Osrm;
use crate::nearest::NearestResponse;
use crate::point::Point;
use crate::route::{RouteRequest, RouteResponse, SimpleRouteResponse};
use crate::tables::{TableAnnotation, TableFallbackCoordinate, TableRequest, TableResponse};
use crate::trip::{TripRequest, TripResponse};

pub struct OsrmEngine {
    instance: Osrm,
}

impl OsrmEngine {
    pub fn new(base_path: &str, algorithm: algorithm::Algorithm) -> Result<Self, OsrmError> {
        let osrm = Osrm::new(base_path, algorithm.as_str())
            .map_err(|e| OsrmError::Native(NativeOsrmError::Initialization(e)))?;
        Ok(OsrmEngine { instance: osrm })
    }

    pub fn table(&self, table_request: TableRequest) -> Result<TableResponse, OsrmError> {
        // Not using is_empty because the lengths are actually needed for the index
        // arrays below
        let len_sources = table_request.sources.len();
        let len_destinations = table_request.destinations.len();
        if len_sources == 0 || len_destinations == 0 {
            return Err(OsrmError::InvalidTableRequest);
        }
        let sources_index: &[usize] = &(0..(len_sources)).collect::<Vec<usize>>()[..];
        let destination_index: &[usize] =
            &(len_sources..(len_sources + len_destinations)).collect::<Vec<usize>>()[..];
        let coordinates: &[(f64, f64)] = &[table_request.sources, table_request.destinations]
            .concat()
            .iter()
            .map(|s| (s.longitude(), s.latitude()))
            .collect::<Vec<(f64, f64)>>()[..];

        // For consistency with osrm, want to ensure we mimic the backend's behaviour
        // when speed should not be specified and the same for the scale factor. Hence,
        // we set them to zero which tells the wrapper to not set the values and use
        // osrm's built in defaults
        let (fallback_coordinate_type, fallback_speed) = match (
            table_request.fallback_coordinate,
            table_request.fallback_speed,
        ) {
            (Some(coord), Some(speed)) => (coord, speed),
            (None, None) => (TableFallbackCoordinate::Input, 0.0),
            _ => return Err(OsrmError::InvalidTableRequest),
        };
        let scale_factor = match (table_request.scale_factor, table_request.annotations) {
            (Some(scale), TableAnnotation::All | TableAnnotation::Duration) => scale,
            (Some(_), TableAnnotation::None | TableAnnotation::Distance) => {
                return Err(OsrmError::InvalidTableRequest);
            }
            (None, _) => 0.0,
        };

        let result = self
            .instance
            .table(
                coordinates,
                Some(sources_index),
                Some(destination_index),
                table_request.annotations,
                fallback_speed,
                fallback_coordinate_type,
                scale_factor,
            )
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        serde_json::from_str::<TableResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))
    }

    pub fn route(&self, route_request: &RouteRequest) -> Result<RouteResponse, OsrmError> {
        let len = route_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidRouteRequest);
        }
        let result = self
            .instance
            .route(route_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        serde_json::from_str::<RouteResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))
    }

    pub fn trip(&self, trip_request: TripRequest) -> Result<TripResponse, OsrmError> {
        let len = trip_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidTripRequest);
        }
        let coordinates: &[(f64, f64)] = &trip_request
            .points
            .iter()
            .map(|p| (p.longitude(), p.latitude()))
            .collect::<Vec<(f64, f64)>>()[..];
        let result = self
            .instance
            .trip(coordinates)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        serde_json::from_str::<TripResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))
    }

    pub fn simple_route(&self, from: Point, to: Point) -> Result<SimpleRouteResponse, OsrmError> {
        let points = [from, to];
        let request = RouteRequest::new(&points).expect("Route request for simple route is empty");

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
            durations: route_response
                .routes
                .first()
                .unwrap()
                .legs
                .iter()
                .map(|l| l.duration)
                .sum(),
        })
    }

    pub fn nearest(&self, point: &Point, number: u64) -> Result<NearestResponse, OsrmError> {
        let result = self
            .instance
            .nearest(point.longitude(), point.latitude(), number)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        let nearest_response = serde_json::from_str::<NearestResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))?;
        Ok(nearest_response)
    }

    pub fn r#match(&self, match_request: &MatchRequest) -> Result<MatchResponse, OsrmError> {
        if match_request.points.is_empty() {
            return Err(OsrmError::InvalidMatchRequest);
        }

        // Collapsing the if requires an if let chain which requires
        // rustc v1.88
        #[allow(clippy::collapsible_if)]
        if let MatchGapsBehaviour::Split = match_request.gaps {
            if match_request.timestamps.is_none() {
                return Err(OsrmError::InvalidMatchRequest);
            }
        }

        let result = self
            .instance
            .r#match(match_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        let response = serde_json::from_str::<MatchResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))?;

        Ok(response)
    }
}
