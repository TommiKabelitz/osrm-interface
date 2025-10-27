pub mod dotenv;

#[cfg(feature = "native")]
#[allow(dead_code)]
pub fn init_native_engine(dotenv_path: &str) -> osrm_interface::native::OsrmEngine {
    let osrm_map_file = dotenv::load_dotenv_value(dotenv_path, "OSRM_MAP_FILE")
        .expect("Failed to load .env which needs to set OSRM_MAP_FILE for native tests");
    osrm_interface::native::OsrmEngine::new(&osrm_map_file, osrm_interface::Algorithm::MLD)
        .expect("Failed to init native OSRM engine")
}

#[cfg(feature = "remote")]
#[allow(dead_code)]
pub fn init_remote_engine(dotenv_path: &str) -> osrm_interface::remote::OsrmEngine {
    let endpoint = dotenv::load_dotenv_value(dotenv_path, "OSRM_ROUTED_ADDRESS")
        .expect("Failed to load .env which needs to set OSRM_ROUTED_ADDRESS for remote tests");
    osrm_interface::remote::OsrmEngine::new(
        endpoint.to_string(),
        osrm_interface::remote::Profile::Car,
    )
}
