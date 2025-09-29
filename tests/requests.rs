use osrm_interface::{point::Point, route::RouteRequest};

#[test]
fn test_invalid_point() {
    let point = Point::new(-91.0, 0.0);
    assert!(
        point.is_none(),
        "Point should return None for latitude < -90.0"
    );

    let point = Point::new(91.0, 0.0);
    assert!(
        point.is_none(),
        "Point should return None for latitude > 90.0"
    );

    let point = Point::new(0.0, -181.0);
    assert!(
        point.is_none(),
        "Point should return None for longitude < -180.0"
    );

    let point = Point::new(0.0, 181.0);
    assert!(
        point.is_none(),
        "Point should return None for latitude > 180.0"
    );
}

#[test]
fn test_invalid_route_request() {
    let points = [];
    let route_request = RouteRequest::new(&points);
    assert!(
        route_request.is_none(),
        "Request should return None for zero points"
    );

    let points = [Point::new(48.040437, 10.316550).expect("Invalid point")];
    let route_request = RouteRequest::new(&points);
    assert!(
        route_request.is_none(),
        "Request should return None for one point"
    );
}
