mod osrm_engine;
use crate::r#match::{MatchGapsBehaviour, MatchRequest};
use crate::request_types::{GeometryType, OverviewZoom};
use crate::route::RouteRequest;
use crate::tables::{TableAnnotation, TableFallbackCoordinate};
pub use osrm_engine::OsrmEngine;

use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;

const ROUTE_ALTERNATIVES: u8 = 1 << 0;
const ROUTE_STEPS: u8 = 1 << 1;
const ROUTE_ANNOTATIONS: u8 = 1 << 2;
const ROUTE_CONTINUE_STRAIGHT: u8 = 1 << 3;

const MATCH_TIDY: u8 = 1 << 0;
const MATCH_STEPS: u8 = 1 << 1;
const MATCH_ANNOTATIONS: u8 = 1 << 2;

#[repr(C)]
struct OsrmResult {
    code: i32,
    message: *mut c_char,
}

#[link(name = "osrm_wrapper", kind = "static")]
unsafe extern "C" {
    fn osrm_create(base_path: *const c_char, algorithm: *const c_char) -> *mut c_void;
    fn osrm_destroy(osrm_instance: *mut c_void);
    fn osrm_table(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
        sources: *const usize,
        num_sources: usize,
        destinations: *const usize,
        num_destinations: usize,
        annotations: TableAnnotation,
        fallback_speed: f64,
        fallback_coordinate_type: TableFallbackCoordinate,
        scale_factor: f64,
    ) -> OsrmResult;
    fn osrm_trip(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
    ) -> OsrmResult;

    fn osrm_route(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
        geometry_type: GeometryType,
        overview_zoom: OverviewZoom,
        flags: u8,
    ) -> OsrmResult;

    fn osrm_match(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
        geometry_type: GeometryType,
        overview_zoom: OverviewZoom,
        timestamps: *const u64,
        num_timestamps: usize,
        radiuses: *const f64,
        num_radiuses: usize,
        gaps_type: MatchGapsBehaviour,
        waypoints: *const usize,
        num_waypoints: usize,
        flags: u8,
    ) -> OsrmResult;

    fn osrm_nearest(osrm_instance: *mut c_void, long: f64, lat: f64, number: u64) -> OsrmResult;

    fn osrm_last_error() -> *const c_char;
    fn osrm_free_string(s: *mut c_char);
}

pub(crate) struct Osrm {
    instance: *mut c_void,
}

impl Osrm {
    pub(crate) fn new(base_path: &str, algorithm: &str) -> Result<Self, String> {
        let c_path = CString::new(base_path).map_err(|e| e.to_string())?;
        let c_algorithm = CString::new(algorithm).map_err(|e| e.to_string())?;
        let instance = unsafe { osrm_create(c_path.as_ptr(), c_algorithm.as_ptr()) };

        if instance.is_null() {
            let err_ptr = unsafe { osrm_last_error() };
            let msg = if err_ptr.is_null() {
                "unknown error".to_string()
            } else {
                unsafe { CStr::from_ptr(err_ptr).to_string_lossy().into_owned() }
            };
            Err(format!("Failure to create an OSRM instance: {}", msg))
        } else {
            Ok(Osrm { instance })
        }
    }

    pub(crate) fn trip(&self, coordinates: &[(f64, f64)]) -> Result<String, String> {
        let coords: Vec<f64> = coordinates
            .iter()
            .flat_map(|&(lon, lat)| vec![lon, lat])
            .collect();
        let result = unsafe { osrm_trip(self.instance, coords.as_ptr(), coordinates.len()) };

        let message_ptr = result.message;
        if message_ptr.is_null() {
            return Err("OSRM returned a null message".to_string());
        }

        let c_str = unsafe { CStr::from_ptr(message_ptr) };
        let rust_str = c_str.to_str().map_err(|e| e.to_string())?.to_owned();

        unsafe {
            osrm_free_string(message_ptr);
        }

        if result.code != 0 {
            return Err(format!("OSRM error: {}", rust_str));
        }

        Ok(rust_str)
    }

    pub(crate) fn route(&self, route_request: &RouteRequest) -> Result<String, String> {
        let num_coords = route_request.points.len();
        let coords: Vec<f64> = route_request
            .points
            .iter()
            .flat_map(|p| [p.longitude(), p.latitude()])
            .collect();

        let mut flags: u8 = 0;
        if route_request.alternatives {
            flags |= ROUTE_ALTERNATIVES;
        }
        if route_request.steps {
            flags |= ROUTE_STEPS;
        }
        if route_request.annotations {
            flags |= ROUTE_ANNOTATIONS;
        }
        if route_request.continue_straight {
            flags |= ROUTE_CONTINUE_STRAIGHT;
        }
        let result = unsafe {
            osrm_route(
                self.instance,
                coords.as_ptr(),
                num_coords,
                route_request.geometry,
                route_request.overview,
                flags,
            )
        };

        let message_ptr = result.message;
        if message_ptr.is_null() {
            return Err("OSRM returned a null message".to_string());
        }

        let c_str = unsafe { CStr::from_ptr(message_ptr) };
        let rust_str = c_str.to_str().map_err(|e| e.to_string())?.to_owned();

        unsafe {
            osrm_free_string(message_ptr);
        }

        if result.code != 0 {
            return Err(format!("OSRM error: {}", rust_str));
        }

        Ok(rust_str)
    }

    pub(crate) fn r#match(&self, match_request: &MatchRequest) -> Result<String, String> {
        let num_coords = match_request.points.len();
        let coords: Vec<f64> = match_request
            .points
            .iter()
            .flat_map(|p| [p.longitude(), p.latitude()])
            .collect();

        let mut flags: u8 = 0;
        if match_request.tidy {
            flags |= MATCH_TIDY;
        }
        if match_request.steps {
            flags |= MATCH_STEPS;
        }
        if match_request.annotations {
            flags |= MATCH_ANNOTATIONS;
        }

        let timestamps = match_request.timestamps.unwrap_or(&[]);
        let radiuses = match_request.radiuses.unwrap_or(&[]);
        let waypoints = match_request.waypoints.unwrap_or(&[]);
        let result = unsafe {
            osrm_match(
                self.instance,
                coords.as_ptr(),
                num_coords,
                match_request.geometry,
                match_request.overview,
                timestamps.as_ptr(),
                timestamps.len(),
                radiuses.as_ptr(),
                radiuses.len(),
                match_request.gaps,
                waypoints.as_ptr(),
                waypoints.len(),
                flags,
            )
        };

        let message_ptr = result.message;
        if message_ptr.is_null() {
            return Err("OSRM returned a null message".to_string());
        }

        let c_str = unsafe { CStr::from_ptr(message_ptr) };
        let rust_str = c_str.to_str().map_err(|e| e.to_string())?.to_owned();

        unsafe {
            osrm_free_string(message_ptr);
        }

        if result.code != 0 {
            return Err(format!("OSRM error: {}", rust_str));
        }

        Ok(rust_str)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn table(
        &self,
        coordinates: &[(f64, f64)],
        sources: Option<&[usize]>,
        destinations: Option<&[usize]>,
        annotations: TableAnnotation,
        fallback_speed: f64,
        fallback_coordinate_type: TableFallbackCoordinate,
        scale_factor: f64,
    ) -> Result<String, String> {
        let flat_coords: Vec<f64> = coordinates
            .iter()
            .flat_map(|&(lon, lat)| vec![lon, lat])
            .collect();
        let sources_vec = sources.unwrap_or(&[]).to_vec();
        let dests_vec = destinations.unwrap_or(&[]).to_vec();

        let result = unsafe {
            osrm_table(
                self.instance,
                flat_coords.as_ptr(),
                coordinates.len(),
                sources_vec.as_ptr(),
                sources_vec.len(),
                dests_vec.as_ptr(),
                dests_vec.len(),
                annotations,
                fallback_speed,
                fallback_coordinate_type,
                scale_factor,
            )
        };

        let message_ptr = result.message;
        if message_ptr.is_null() {
            return Err("OSRM returned a null message".to_string());
        }

        let c_str = unsafe { CStr::from_ptr(message_ptr) };
        let rust_str = c_str.to_str().map_err(|e| e.to_string())?.to_owned();

        unsafe {
            osrm_free_string(message_ptr);
        }

        if result.code != 0 {
            return Err(format!("OSRM error: {}", rust_str));
        }

        Ok(rust_str)
    }

    pub(crate) fn nearest(&self, long: f64, lat: f64, number: u64) -> Result<String, String> {
        let result = unsafe { osrm_nearest(self.instance, long, lat, number) };

        let message_ptr = result.message;
        if message_ptr.is_null() {
            return Err("OSRM returned a null message".to_string());
        }

        let c_str = unsafe { CStr::from_ptr(message_ptr) };
        let rust_str = c_str.to_str().map_err(|e| e.to_string())?.to_owned();

        unsafe {
            osrm_free_string(message_ptr);
        }

        if result.code != 0 {
            return Err(format!("OSRM error: {}", rust_str));
        }

        Ok(rust_str)
    }
}

impl Drop for Osrm {
    fn drop(&mut self) {
        unsafe {
            osrm_destroy(self.instance);
        }
    }
}

unsafe impl Send for Osrm {}
unsafe impl Sync for Osrm {}
