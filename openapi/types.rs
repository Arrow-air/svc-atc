use chrono::{DateTime, Utc};
/// Types used for REST communication with the svc-cargo server
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Example Request Body Information Type
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct AckRequest {
    /// Flight plan ID to confirm or reject
    pub fp_id: String,

    /// Acknowledgement Status
    pub status: AckStatus,
}

/// Confirm itinerary Operation Status
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum AckStatus {
    /// Unauthorized request
    Deny,

    /// Unavailable Service
    Confirm,
}
