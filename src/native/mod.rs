mod osrm_engine;
use crate::request_types::{GeometryType, OverviewZoom};
use crate::route::RouteRequest;
pub use osrm_engine::OsrmEngine;

use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;

const ROUTE_ALTERNATIVES: u8 = 1 << 0;
const ROUTE_STEPS: u8 = 1 << 1;
const ROUTE_ANNOTATIONS: u8 = 1 << 2;
const ROUTE_CONTINUE_STRAIGHT: u8 = 1 << 3;

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
            Err("Failure to create an OSRM instance.".to_string())
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

    pub(crate) fn table(
        &self,
        coordinates: &[(f64, f64)],
        sources: Option<&[usize]>,
        destinations: Option<&[usize]>,
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
