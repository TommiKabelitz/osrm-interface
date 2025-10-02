use itertools::Itertools;

use crate::errors::{OsrmError, RemoteOsrmError};
use crate::point::Point;
use crate::request_types::Profile;
use crate::route::{RouteRequest, SimpleRouteResponse};
use crate::service_responses::{RouteResponse, TableResponse, TripResponse};
use crate::tables::TableRequest;
use crate::trip::TripRequest;

pub struct OsrmEngine {
    endpoint: String,
    pub profile: Profile,
}

impl OsrmEngine {
    pub fn new(endpoint: String, profile: Profile) -> Self {
        Self { endpoint, profile }
    }

    pub fn table(&self, table_request: TableRequest) -> Result<TableResponse, OsrmError> {
        let len_sources = table_request.sources.len();
        let len_destinations = table_request.destinations.len();
        if len_sources == 0 || len_destinations == 0 {
            return Err(OsrmError::InvalidTableRequest);
        }

        let coordinates = table_request
            .sources
            .iter()
            .chain(table_request.destinations.iter())
            .map(|p| format!("{:.6},{:.6}", p.longitude(), p.latitude()))
            .join(";");

        let source_indices = (0..len_sources).map(|i| format!("{}", i)).join(",");
        let destination_indices = (len_sources..(len_sources + len_destinations))
            .map(|i| format!("{}", i))
            .join(";");

        let url = format!(
            "{}/table/v1/{}/{coordinates}?sources={source_indices}&destinations={destination_indices}",
            self.endpoint,
            self.profile.url_form()
        );
        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?
            .into_body()
            .read_json::<TableResponse>()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?;

        Ok(response)
    }

    pub fn route(&self, route_request: &RouteRequest) -> Result<RouteResponse, OsrmError> {
        let len = route_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidRouteRequest);
        }
        let coordinates = route_request
            .points
            .iter()
            .map(|p| format!("{:.6},{:.6}", p.longitude(), p.latitude()))
            .join(";");

        let url = format!(
            "{}/route/v1/{}/{coordinates}?alternatives={}&steps={}&geometries={}&overview={}&annotations={}",
            self.endpoint,
            self.profile.url_form(),
            route_request.alternatives,
            route_request.steps,
            route_request.geometry.url_form(),
            route_request.overview.url_form(),
            route_request.annotations
        );
        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?
            .into_body()
            .read_to_string()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?;

        serde_json::from_str::<RouteResponse>(&response)
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))
    }

    pub fn trip(&self, trip_request: &TripRequest) -> Result<TripResponse, OsrmError> {
        let len = trip_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidTripRequest);
        }
        let coordinates = trip_request
            .points
            .iter()
            .map(|p| format!("{:.6},{:.6}", p.longitude(), p.latitude()))
            .join(";");

        let url = format!(
            "{}/trip/v1/{}/{coordinates}?steps={}&geometries={}&overview={}&annotations={}",
            self.endpoint,
            self.profile.url_form(),
            trip_request.steps,
            trip_request.geometry.url_form(),
            trip_request.overview.url_form(),
            trip_request.annotations
        );
        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?
            .into_body()
            .read_to_string()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?;

        serde_json::from_str::<TripResponse>(&response)
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))
    }

    pub fn simple_route(&self, from: Point, to: Point) -> Result<SimpleRouteResponse, OsrmError> {
        let points = [from, to];
        let full_request =
            RouteRequest::new(&points).expect("Route request for simple route is empty");
        let response = self.route(&full_request)?;

        Ok(SimpleRouteResponse {
            code: response.code,
            distance: response
                .routes
                .first()
                .unwrap()
                .legs
                .iter()
                .map(|l| l.distance)
                .sum(),
            durations: response
                .routes
                .first()
                .unwrap()
                .legs
                .iter()
                .map(|l| l.duration)
                .sum(),
        })
    }
}
