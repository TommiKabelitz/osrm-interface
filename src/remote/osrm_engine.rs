use itertools::Itertools;

use crate::Point;
use crate::errors::{OsrmError, RemoteOsrmError};
use crate::r#match::{MatchRequest, MatchResponse};
use crate::nearest::{NearestRequest, NearestResponse};
use crate::request_types::{Exclude, Profile};
use crate::route::{RouteRequest, RouteRequestBuilder, RouteResponse, SimpleRouteResponse};
use crate::table::{TableAnnotation, TableRequest, TableResponse};
use crate::trip::{TripRequest, TripResponse};

/// The engine for calling into osrm-backend through the HTTP web API.
#[cfg_attr(doc, doc(cfg(feature = "remote")))]
pub struct OsrmEngine {
    endpoint: String,
    pub profile: Profile,
}

impl OsrmEngine {
    /// Initialise the remote engine.
    ///
    /// `profile` is a required argument because it is used by Project OSRM
    /// when you route to their official hosted version of the HTTP web API in
    /// the url path to route using the proper profile. When running `osrm-routed`
    /// manually, the profile is still required in the url, but is ignored by
    /// osrm-routed. The profile used is simply that of the map data that `osrm-routed`
    /// is using.
    ///
    /// See the module level documentation for more information about profiles.
    pub fn new(endpoint: String, profile: Profile) -> Self {
        Self { endpoint, profile }
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
            "{}/table/v1/{}/{coordinates}?sources={source_indices}&destinations={destination_indices}&generate_hints={}",
            self.endpoint,
            self.profile.url_form(),
            table_request.generate_hints
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

        // I know this is repetitive, but I don't really care. Defining a
        // generic function feels like overkill and I can't be bothered with
        // a closure either
        let mut bearing_string = String::new();
        let mut first = true;
        if let Some(source_bearings) = table_request.source_bearings {
            for b in source_bearings {
                if !first {
                    bearing_string.push(';');
                }
                if let Some(b) = b {
                    bearing_string.push_str(&b.url_form());
                }
                first = false;
            }
        } else {
            for _ in 0..len_sources {
                if !first {
                    bearing_string.push(';');
                }
                first = false;
            }
        }
        if let Some(destination_bearings) = table_request.destination_bearings {
            for b in destination_bearings {
                if !first {
                    bearing_string.push(';');
                }
                if let Some(b) = b {
                    bearing_string.push_str(&b.url_form());
                }
                first = false;
            }
        } else {
            for _ in 0..len_destinations {
                if !first {
                    bearing_string.push(';');
                }
                first = false;
            }
        }
        if table_request.source_bearings.is_some() || table_request.destination_bearings.is_some() {
            url.push_str(&format!("&bearings={}", bearing_string));
        }

        let mut first = true;
        let mut radius_string = String::new();
        if let Some(source_radiuses) = table_request.source_radiuses {
            for r in source_radiuses {
                if !first {
                    radius_string.push(';');
                }
                if let Some(r) = r {
                    radius_string.push_str(&format!("{r:.12}"));
                }
                first = false;
            }
        } else {
            for _ in 0..len_sources {
                if !first {
                    radius_string.push(';');
                }
                first = false;
            }
        }
        if let Some(destination_radiuses) = table_request.destination_radiuses {
            for r in destination_radiuses {
                if !first {
                    radius_string.push(';');
                }
                if let Some(r) = r {
                    radius_string.push_str(&format!("{r:.12}"));
                }
                first = false;
            }
        } else {
            for _ in 0..len_destinations {
                if !first {
                    radius_string.push(';');
                }
                first = false;
            }
        }
        if table_request.source_radiuses.is_some() || table_request.destination_radiuses.is_some() {
            url.push_str(&format!("&radiuses={}", radius_string));
        }

        let mut first = true;
        let mut hints_string = String::new();
        if let Some(source_hints) = table_request.source_hints {
            for h in source_hints {
                if !first {
                    hints_string.push(';');
                }
                if let Some(h) = h {
                    hints_string.push_str(h);
                }
                first = false;
            }
        } else {
            for _ in 0..len_sources {
                if !first {
                    hints_string.push(';');
                }
                first = false;
            }
        }
        if let Some(destination_hints) = table_request.destination_hints {
            for h in destination_hints {
                if !first {
                    hints_string.push(';');
                }
                if let Some(h) = h {
                    hints_string.push_str(h);
                }
                first = false;
            }
        } else {
            for _ in 0..len_destinations {
                if !first {
                    hints_string.push(';');
                }
                first = false;
            }
        }
        if table_request.source_hints.is_some() || table_request.destination_hints.is_some() {
            url.push_str(&format!("&hints={}", hints_string));
        }

        let mut first = true;
        let mut approaches_string = String::new();
        if let Some(source_approaches) = table_request.source_approaches {
            for a in source_approaches {
                if !first {
                    approaches_string.push(';');
                }
                approaches_string.push_str(a.url_form());
                first = false;
            }
        } else {
            for _ in 0..len_sources {
                if !first {
                    approaches_string.push(';');
                }
                first = false;
            }
        }
        if let Some(destination_approaches) = table_request.destination_approaches {
            for a in destination_approaches {
                if !first {
                    approaches_string.push(';');
                }
                approaches_string.push_str(a.url_form());
                first = false;
            }
        } else {
            for _ in 0..len_destinations {
                if !first {
                    approaches_string.push(';');
                }
                first = false;
            }
        }
        if table_request.source_approaches.is_some()
            || table_request.destination_approaches.is_some()
        {
            url.push_str(&format!("&approaches={}", approaches_string));
        }

        if let Some(exclude) = table_request.exclude {
            let exclude = exclude
                .iter()
                .map(|exclude| match &exclude {
                    Exclude::Bicycle(v) => v.as_str(),
                    Exclude::Car(v) => v.as_str(),
                })
                .join(",");
            url.push_str(&format!("&exclude={}", exclude));
        }
        if let Some(snapping) = table_request.snapping {
            url.push_str(&format!("&snapping={}", snapping.url_form()));
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

    /// Given an ordered set of `Point`s or `Hint`s, route through those points in the
    /// given order.
    ///
    /// See `RouteRequest` for all possible options.
    ///
    /// ## Official documentation
    ///
    /// Finds the fastest route between coordinates in the supplied order.
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

        let mut url = format!(
            "{}/route/v1/{}/{coordinates}?alternatives={}&steps={}&geometries={}&overview={}&annotations={}&generate_hints={}&skip_waypoints={}",
            self.endpoint,
            self.profile.url_form(),
            route_request.alternatives,
            route_request.steps,
            route_request.geometry.url_form(),
            route_request.overview.url_form(),
            route_request.annotations,
            route_request.generate_hints,
            route_request.skip_waypoints,
        );
        if let Some(bearings) = route_request.bearings {
            let bearings = bearings
                .iter()
                .map(|bearing| {
                    if let Some(b) = bearing {
                        b.url_form()
                    } else {
                        String::new()
                    }
                })
                .join(";");
            url.push_str(&format!("&bearings={}", bearings));
        }
        if let Some(radiuses) = route_request.radiuses {
            let radiuses = radiuses
                .iter()
                .map(|r| {
                    if let Some(r) = r {
                        format!("{r:.12}")
                    } else {
                        String::new()
                    }
                })
                .join(";");
            url.push_str(&format!("&radiuses={}", radiuses));
        }

        if let Some(hints) = route_request.hints {
            let hints = hints.iter().map(|hint| hint.unwrap_or("")).join(";");
            url.push_str(&format!("&hints={}", hints));
        }

        if let Some(approaches) = route_request.approaches {
            let approaches = approaches
                .iter()
                .map(|approach| approach.url_form())
                .join(";");
            url.push_str(&format!("&approaches={}", approaches));
        }

        if let Some(exclude) = route_request.exclude {
            let exclude = exclude
                .iter()
                .map(|exclude| match &exclude {
                    Exclude::Bicycle(v) => v.as_str(),
                    Exclude::Car(v) => v.as_str(),
                })
                .join(",");
            url.push_str(&format!("&exclude={}", exclude));
        }
        if let Some(snapping) = route_request.snapping {
            url.push_str(&format!("&snapping={}", snapping.url_form()));
        }
        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?
            .into_body()
            .read_to_string()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?;
        serde_json::from_str::<RouteResponse>(&response)
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))
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
        let len = trip_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidTripRequest);
        }
        let coordinates = trip_request
            .points
            .iter()
            .map(|p| format!("{:.6},{:.6}", p.longitude(), p.latitude()))
            .join(";");

        let mut url = format!(
            "{}/trip/v1/{}/{coordinates}?steps={}&geometries={}&overview={}&annotations={}&roundtrip={}&source={}&destination={}&generate_hints={}&skip_waypoints={}",
            self.endpoint,
            self.profile.url_form(),
            trip_request.steps,
            trip_request.geometry.url_form(),
            trip_request.overview.url_form(),
            trip_request.annotations,
            trip_request.roundtrip,
            trip_request.source.url_form(),
            trip_request.destination.url_form(),
            trip_request.generate_hints,
            trip_request.skip_waypoints,
        );

        if let Some(bearings) = trip_request.bearings {
            let bearings = bearings
                .iter()
                .map(|bearing| {
                    if let Some(b) = bearing {
                        b.url_form()
                    } else {
                        String::new()
                    }
                })
                .join(";");
            url.push_str(&format!("&bearings={}", bearings));
        }
        if let Some(radiuses) = trip_request.radiuses {
            let radiuses = radiuses
                .iter()
                .map(|r| {
                    if let Some(r) = r {
                        format!("{r:.12}")
                    } else {
                        String::new()
                    }
                })
                .join(";");
            url.push_str(&format!("&radiuses={}", radiuses));
        }

        if let Some(hints) = trip_request.hints {
            let hints = hints.iter().map(|hint| hint.unwrap_or("")).join(";");
            url.push_str(&format!("&hints={}", hints));
        }

        if let Some(approaches) = trip_request.approaches {
            let approaches = approaches
                .iter()
                .map(|approach| approach.url_form())
                .join(";");
            url.push_str(&format!("&approaches={}", approaches));
        }

        if let Some(exclude) = trip_request.exclude {
            let exclude = exclude
                .iter()
                .map(|exclude| match &exclude {
                    Exclude::Bicycle(v) => v.as_str(),
                    Exclude::Car(v) => v.as_str(),
                })
                .join(",");
            url.push_str(&format!("&exclude={}", exclude));
        }
        if let Some(snapping) = trip_request.snapping {
            url.push_str(&format!("&snapping={}", snapping.url_form()));
        }

        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?
            .into_body()
            .read_to_string()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?;
        serde_json::from_str::<TripResponse>(&response)
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))
    }

    /// A massively simplified interface for routing just between two points.
    ///
    /// Calls OsrmEngine::route with default options.
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
        let mut url = format!(
            "{}/nearest/v1/{}/{:.6},{:.6}?number={}",
            self.endpoint,
            self.profile.url_form(),
            nearest_request.point.longitude(),
            nearest_request.point.latitude(),
            nearest_request.number,
        );
        if let Some(bearing) = nearest_request.bearing {
            url.push_str(&format!("&bearings={}", bearing.url_form()));
        }
        if let Some(radius) = nearest_request.radius {
            url.push_str(&format!("&radiuses={:.12}", radius));
        }
        if let Some(approach) = nearest_request.approach {
            url.push_str(&format!("&approaches={}", approach.url_form()));
        }
        if let Some(exclude) = nearest_request.exclude {
            let exclude = exclude
                .iter()
                .map(|exclude| match &exclude {
                    Exclude::Bicycle(v) => v.as_str(),
                    Exclude::Car(v) => v.as_str(),
                })
                .join(",");
            url.push_str(&format!("&exclude={}", exclude));
        }
        if let Some(snapping) = nearest_request.snapping {
            url.push_str(&format!("&snapping={}", snapping.url_form()));
        }

        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?
            .into_body()
            .read_to_string()
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))?;
        serde_json::from_str::<NearestResponse>(&response)
            .map_err(|e| OsrmError::Remote(RemoteOsrmError::EndpointError(e.to_string())))
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
        if match_request.points.is_empty() {
            return Err(OsrmError::InvalidMatchRequest);
        }

        let coordinates = match_request
            .points
            .iter()
            .map(|p| format!("{:.6},{:.6}", p.longitude(), p.latitude()))
            .join(";");

        let mut url = format!(
            "{}/match/v1/{}/{coordinates}?steps={}&geometries={}&overview={}&annotations={}&gaps={}&tidy={}&generate_hints={}&skip_waypoints={}",
            self.endpoint,
            self.profile.url_form(),
            match_request.steps,
            match_request.geometry.url_form(),
            match_request.overview.url_form(),
            match_request.annotations,
            match_request.gaps.url_form(),
            match_request.tidy,
            match_request.generate_hints,
            match_request.skip_waypoints,
        );

        if let Some(timestamps) = match_request.timestamps {
            let timestamps = timestamps.iter().map(|t| format!("{t}")).join(";");
            url.push_str(&format!("&timestamps={}", timestamps));
        }
        if let Some(waypoints) = match_request.waypoints {
            let waypoints = waypoints.iter().map(|w| format!("{w}")).join(";");
            url.push_str(&format!("&waypoints={}", waypoints));
        }
        if let Some(bearings) = match_request.bearings {
            let bearings = bearings
                .iter()
                .map(|bearing| {
                    if let Some(b) = bearing {
                        b.url_form()
                    } else {
                        String::new()
                    }
                })
                .join(";");
            url.push_str(&format!("&bearings={}", bearings));
        }
        if let Some(radiuses) = match_request.radiuses {
            let radiuses = radiuses
                .iter()
                .map(|r| {
                    if let Some(r) = r {
                        format!("{r:.12}")
                    } else {
                        String::new()
                    }
                })
                .join(";");
            url.push_str(&format!("&radiuses={}", radiuses));
        }

        if let Some(hints) = match_request.hints {
            let hints = hints.iter().map(|hint| hint.unwrap_or("")).join(";");
            url.push_str(&format!("&hints={}", hints));
        }

        if let Some(approaches) = match_request.approaches {
            let approaches = approaches
                .iter()
                .map(|approach| approach.url_form())
                .join(";");
            url.push_str(&format!("&approaches={}", approaches));
        }

        if let Some(exclude) = match_request.exclude {
            let exclude = exclude
                .iter()
                .map(|exclude| match &exclude {
                    Exclude::Bicycle(v) => v.as_str(),
                    Exclude::Car(v) => v.as_str(),
                })
                .join(",");
            url.push_str(&format!("&exclude={}", exclude));
        }
        if let Some(snapping) = match_request.snapping {
            url.push_str(&format!("&snapping={}", snapping.url_form()));
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
