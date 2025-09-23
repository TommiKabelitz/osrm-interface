#[derive(Debug, Clone)]
pub struct Point {
    latitude: f64,
    longitude: f64,
}

impl Point {
    pub fn new(latitude: f64, longitude: f64) -> Option<Self> {
        // Range contains produces the same assembly as chained <= and >= with optimisation
        if !((-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude)) {
            return None;
        }
        Some(Self {
            latitude,
            longitude,
        })
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }
}
