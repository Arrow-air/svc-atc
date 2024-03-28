//! Common code for GRPC and REST servers

#[macro_use]
pub mod macros;

use crate::grpc::client::GrpcClients;
use chrono::Utc;
use std::fmt;
use svc_storage_client_grpc::prelude::*;

/// Error type for ack_flight
#[derive(Debug, Copy, Clone)]
pub enum AckError {
    /// Internal Error
    Internal,

    /// Dependencies not available
    Unavailable,
}

impl fmt::Display for AckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AckError::Internal => write!(f, "Internal Error"),
            AckError::Unavailable => write!(f, "Dependencies not available"),
            // AckError::Unauthorized => write!(f, "Unauthorized request"),
        }
    }
}

/// This request might come in over REST, or through GRPC someday
///  if another microservice has a software-hardware link to a radio antenna
pub async fn ack_flight(fp_id: String, grpc_clients: &GrpcClients) -> Result<(), AckError> {
    let client = &grpc_clients.storage.flight_plan;

    let result = client
        .get_by_id(Id { id: fp_id.clone() })
        .await
        .map_err(|e| {
            common_error!("(ack_flight) {}", e);
            AckError::Internal
        })?;

    let mut data = result.into_inner().data.ok_or_else(|| {
        common_error!("(ack_flight) Couldn't get data from object id: {}", fp_id);
        AckError::Internal
    })?;

    data.carrier_ack = Some(Utc::now().into());

    //
    // Update the flight plan record to show that it has been acknowledged
    //
    let request = flight_plan::UpdateObject {
        id: fp_id,
        data: Some(data),
        mask: Some(FieldMask {
            paths: vec!["carrier_ack".to_string()],
        }),
    };

    //
    // TODO(R4) - Push to queue and retry on failure
    //
    client.update(request).await.map_err(|e| {
        let error_msg = "svc-storage failure.".to_string();
        common_error!("(acknowledge_flight_plan) {}: {e}", &error_msg);
        AckError::Internal
    })?;

    Ok(())
}
