#[macro_use]
pub mod macros;

use crate::grpc::client::GrpcClients;
use chrono::Utc;
use lib_common::time::datetime_to_timestamp;
use prost_types::FieldMask;
use std::fmt;
use svc_storage_client_grpc::resources::flight_plan::Data as FpData;
use svc_storage_client_grpc::resources::flight_plan::UpdateObject as FpUpdate;
use svc_storage_client_grpc::ClientConnect;

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
    //
    // TODO(R4) - Check that it came from authenticated source
    //

    //
    // Update the flight plan record to show that it has been acknowledged
    //
    let Some(now) = datetime_to_timestamp(&Utc::now()) else {
        let error_msg = "Failed to convert timestamp.".to_string();
        // rest_error!("(acknowledge_flight_plan) {}", &error_msg);
        return Err(AckError::Internal);
    };

    let request = tonic::Request::new(FpUpdate {
        id: fp_id,
        data: Some(FpData {
            carrier_ack: Some(Utc::now()),
        }),
        mask: Some(FieldMask {
            paths: vec!["carrier_ack".to_string()],
        }),
    });

    let Ok(mut client) = grpc_clients.storage.flight_plan.get_client().await else {
        let error_msg = "svc-storage unavailable.".to_string();
        // rest_error!("(acknowledge_flight_plan) {}", &error_msg);
        return Err(AckError::Unavailable);
    };

    //
    // TODO(R4) - Push to queue and retry on failure
    //
    let Ok(response) = client.update(request).await else {
        let error_msg = "svc-storage failure.".to_string();
        // rest_error!("(acknowledge_flight_plan) {}", &error_msg);
        return Err(AckError::Internal);
    };

    Ok(())
}
