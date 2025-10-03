#![cfg(all(feature = "native", feature = "remote"))]

mod common;
use common::{init_native_engine, init_remote_engine};

use osrm_interface::{point::Point, route::RouteRequest};
use rand::Rng;

#[test]
fn test_native_and_remote_route() {
    let native_engine = init_native_engine(".env");
    let remote_engine = init_remote_engine(".env");

    let mut rng = rand::rng();
    let points = (0..10)
        .map(|_| Point::new(rng.random_range(49.0..53.0), rng.random_range(8.3..12.0)).unwrap())
        .collect::<Vec<_>>();

    let route_request = RouteRequest::new(&points).expect("No points in request");

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
