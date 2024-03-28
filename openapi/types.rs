/// Types used for REST communication with the svc-cargo server
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use chrono::{DateTime, Utc};

/// Example Request Body Information Type
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct AckRequest {
    /// Flight plan ID to confirm or reject
    pub fp_id: String,

    /// Acknowledgement Status
    pub status: AckStatus,
}

/// Confirm itinerary Operation Status
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum AckStatus {
    /// Unauthorized request
    Deny,

    /// Unavailable Service
    Confirm,
}

/// Latitude, longitude, and altitude
///  following the WGS-84 standard
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub struct PointZ {
    /// Latitude
    pub latitude: f64,

    /// Longitude
    pub longitude: f64,

    /// Altitude in meters
    pub altitude_meters: f64
}

/// Flight Plan Information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FlightPlan {
    /// Flight ID
    pub flight_uuid: String,

    /// Session ID
    pub session_id: String,

    /// Vehicle ID
    pub aircraft_id: String,

    /// Origin Vertiport ID
    pub origin_vertiport_id: String,

    /// Target Vertiport ID
    pub target_vertiport_id: String,

    /// Origin Vertipad ID
    pub origin_vertipad_id: String,

    /// Target Vertipad ID
    pub target_vertipad_id: String,

    /// Origin Time Start
    pub origin_timeslot_start: DateTime<Utc>,

    /// Origin Time End
    pub origin_timeslot_end: DateTime<Utc>,

    /// Target Time Start
    pub target_timeslot_start: DateTime<Utc>,

    /// Target Time End
    pub target_timeslot_end: DateTime<Utc>,

    /// Path
    pub path: Vec<PointZ>
}
