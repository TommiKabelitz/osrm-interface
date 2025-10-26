//! Mock version of the OSRM engine. Produces output of the correct type with
//! fabricated data for the sake of development when the backend is unavailable.

mod osrm_engine;
pub use osrm_engine::OsrmEngine;
