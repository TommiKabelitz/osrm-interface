use itertools::Itertools;

use crate::errors::{OsrmError, RemoteOsrmError};
use crate::r#match::{MatchGapsBehaviour, MatchRequest, MatchResponse};
use crate::nearest::NearestResponse;
use crate::point::Point;
use crate::request_types::Profile;
use crate::route::{RouteRequest, RouteRequestBuilder, RouteResponse, SimpleRouteResponse};
use crate::table::{TableAnnotation, TableRequest, TableResponse};
use crate::trip::{TripRequest, TripResponse};

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

        let source_indices = (0..len_sources).map(|i| format!("{}", i)).join(";");
        let destination_indices = (len_sources..(len_sources + len_destinations))
            .map(|i| format!("{}", i))
            .join(";");

        let mut url = format!(
            "{}/table/v1/{}/{coordinates}?sources={source_indices}&destinations={destination_indices}",
            self.endpoint,
            self.profile.url_form(),
        );

        match (
            table_request.fallback_coordinate,
            table_request.fallback_speed,
        ) {
            (Some(coord), Some(speed)) => url.push_str(&format!(
                "&fallback_speed={}&fallback_coordinate={}",
                speed,
                coord.url_form()
            )),
            (None, None) => (),
            _ => return Err(OsrmError::InvalidTableRequest),
        }
        match (table_request.scale_factor, table_request.annotations) {
            (Some(scale), TableAnnotation::All | TableAnnotation::Duration) => {
                url.push_str(&format!(
                    "&annotations={}&scale_factor={}",
                    table_request.annotations.url_form(),
                    scale
                ))
            }
            (Some(_), _) => return Err(OsrmError::InvalidTableRequest),
            (None, annotations) => {
                url.push_str(&format!("&annotations={}", annotations.url_form()))
            }
        }
        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?
            .into_body()
            .read_to_string()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?;
        serde_json::from_str::<TableResponse>(&response)
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))
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
        let full_request = RouteRequestBuilder::new(&points)
            .build()
            .expect("Route request for simple route is empty");
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

    pub fn nearest(&self, point: &Point, number: u64) -> Result<NearestResponse, OsrmError> {
        let url = format!(
            "{}/nearest/v1/{}/{:.6},{:.6}?number={}",
            self.endpoint,
            self.profile.url_form(),
            point.longitude(),
            point.latitude(),
            number,
        );
        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?
            .into_body()
            .read_to_string()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?;
        serde_json::from_str::<NearestResponse>(&response)
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))
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
        let coordinates = match_request
            .points
            .iter()
            .map(|p| format!("{:.6},{:.6}", p.longitude(), p.latitude()))
            .join(";");

        let mut url = format!(
            "{}/match/v1/{}/{coordinates}?steps={}&geometries={}&overview={}&annotations={}&gaps={}&tidy={}",
            self.endpoint,
            self.profile.url_form(),
            match_request.steps,
            match_request.geometry.url_form(),
            match_request.overview.url_form(),
            match_request.annotations,
            match_request.gaps.url_form(),
            match_request.tidy
        );

        if let Some(timestamps) = match_request.timestamps {
            let timestamps = timestamps.iter().map(|t| format!("{t}")).join(";");
            url.push_str(&format!("&timestamps={}", timestamps));
        }
        if let Some(radiuses) = match_request.radiuses {
            let radiuses = radiuses.iter().map(|r| format!("{r:.12}")).join(";");
            url.push_str(&format!("&radiuses={}", radiuses));
        }
        if let Some(waypoints) = match_request.waypoints {
            let waypoints = waypoints.iter().map(|w| format!("{w}")).join(";");
            url.push_str(&format!("&waypoints={}", waypoints));
        }

        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?
            .into_body()
            .read_to_string()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?;
        serde_json::from_str::<MatchResponse>(&response)
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))
    }
}
