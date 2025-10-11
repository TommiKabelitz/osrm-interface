#![cfg(all(feature = "native", feature = "remote"))]

mod common;
use common::{init_native_engine, init_remote_engine};

use osrm_interface::{
    r#match::MatchRequestBuilder,
    osrm_response_types::Geometry,
    point::Point,
    request_types::{GeometryType, OverviewZoom},
    route::RouteRequestBuilder,
};
use rand::Rng;

#[test]
fn test_native_and_remote_route() {
    let native_engine = init_native_engine(".env");
    let remote_engine = init_remote_engine(".env");

    let mut rng = rand::rng();
    let points = (0..10)
        .map(|_| Point::new(rng.random_range(49.0..53.0), rng.random_range(8.3..12.0)).unwrap())
        .collect::<Vec<_>>();

    let route_request = RouteRequestBuilder::new(&points)
        .build()
        .expect("No points in request");

    let native_response = native_engine
        .route(&route_request)
        .expect("Failed to route request");

    let remote_response = remote_engine
        .route(&route_request)
        .expect("Failed to route request");

    assert_eq!(
        native_response.code, "Ok",
        "Native response code is not 'Ok'"
    );
    assert_eq!(
        remote_response.code, "Ok",
        "Remote response code is not 'Ok'"
    );
    assert_eq!(
        native_response
            .routes
            .iter()
            .map(|r| r.legs.len())
            .sum::<usize>(),
        remote_response
            .routes
            .iter()
            .map(|r| r.legs.len())
            .sum::<usize>(),
        "Route response lengths disagree"
    );
    assert_eq!(
        native_response.waypoints.len(),
        remote_response.waypoints.len(),
        "Different numbers of waypoints",
    );

    assert!(
        (native_response
            .routes
            .iter()
            .map(|r| r.duration)
            .sum::<f64>() as i64
            - remote_response
                .routes
                .iter()
                .map(|r| r.duration)
                .sum::<f64>() as i64)
            .abs()
            < 5,
        "Durations differ by more than 5 seconds"
    );
}

#[test]
fn test_compare_nearest() {
    let remote_engine = init_remote_engine(".env");
    let native_engine = init_native_engine(".env");
    let num_points = 3;

    let point = Point::new(48.040437, 10.316550).expect("Invalid point");
    let remote_response = remote_engine
        .nearest(&point, num_points)
        .expect("Failed to find nearest");
    let native_response = native_engine
        .nearest(&point, num_points)
        .expect("Failed to find nearest");

    assert_eq!(
        remote_response.code, native_response.code,
        "Responses returned different codes"
    );
    assert_eq!(
        remote_response.waypoints.len(),
        native_response.waypoints.len(),
        "Responses returned different number of waypoints"
    );

    assert!(
        remote_response
            .waypoints
            .iter()
            .zip(native_response.waypoints.iter())
            .any(|(w_r, w_n)| {
                (w_r.location[0] - w_n.location[0]).abs() < 1e-6
                    && (w_r.location[1] - w_n.location[1]).abs() < 1e-6
            }),
        "Responses have different snapped locations"
    )
}

#[test]
fn test_compare_match() {
    let remote_engine = init_remote_engine(".env");
    let native_engine = init_native_engine(".env");

    let points = [
        Point::new(51.097683869065804, 11.517827906178626).expect("Invalid point"),
        Point::new(51.098737989249116, 11.526971690952534).expect("Invalid point"),
        Point::new(51.09937599770893, 11.530571780087442).expect("Invalid point"),
        Point::new(51.099195691869646, 11.535806265573953).expect("Invalid point"),
        Point::new(51.09883507808152, 11.541924208526543).expect("Invalid point"),
        Point::new(51.10019429998817, 11.547070348266445).expect("Invalid point"),
        Point::new(51.10187286817584, 11.549241921250635).expect("Invalid point"),
        Point::new(51.10307204569769, 11.561966320703071).expect("Invalid point"),
    ];

    let match_request = MatchRequestBuilder::new(&points)
        .geometry(GeometryType::Polyline)
        .overview(OverviewZoom::Full)
        .gaps(osrm_interface::r#match::MatchGapsBehaviour::Ignore)
        .build()
        .expect("Failed to create match request");
    let remote_response = remote_engine
        .r#match(&match_request)
        .expect("Failed to match route");
    let native_response = native_engine
        .r#match(&match_request)
        .expect("Failed to match route");

    assert_eq!(
        remote_response.code, "Ok",
        "Remote response code is not 'Ok'"
    );
    assert_eq!(
        native_response.code, "Ok",
        "Native response code is not 'Ok'"
    );
    let remote_polyline = if let Some(Geometry::Polyline(line)) = &remote_response
        .matchings
        .first()
        .expect("No matching routes in remote response")
        .geometry
    {
        line
    } else {
        panic!("Geometry should be polyline")
    };
    let native_polyline = if let Some(Geometry::Polyline(line)) = &native_response
        .matchings
        .first()
        .expect("No matching routes in remote response")
        .geometry
    {
        line
    } else {
        panic!("Geometry should be polyline")
    };
    assert_eq!(
        remote_polyline, native_polyline,
        "Polylines are not the same"
    );
}
