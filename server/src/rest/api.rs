//! Rest API implementations

use crate::grpc::client::GrpcClients;
use axum::{extract::Extension, Json};
use hyper::StatusCode;

use prost_types::FieldMask;
use svc_storage_client_grpc::resources::flight_plan::UpdateObject as FpUpdate;

/// openapi generated rest types
pub mod rest_types {
    include!("../../../openapi/types.rs");
}

// GRPC client types
// use svc_scheduler_client_grpc::grpc::{
//     ConfirmItineraryRequest, Id, Itinerary as SchedulerItinerary, QueryFlightPlan,
// };

// REST types the caller will use
pub use rest_types::{AckRequest, AckStatus};

/// Provides a way to tell a caller if the service is healthy.
/// Checks dependencies, making sure all connections can be made.
#[utoipa::path(
    get,
    path = "/health",
    tag = "svc-atc",
    responses(
        (status = 200, description = "Service is healthy, all dependencies running."),
        (status = 503, description = "Service is unhealthy, one or more dependencies unavailable.")
    )
)]
pub async fn health_check(
    Extension(mut grpc_clients): Extension<GrpcClients>,
) -> Result<(), StatusCode> {
    rest_debug!("(health_check) entry.");

    let ok = true;

    let result = grpc_clients.storage.flight_plan.get_client().await;
    if result.is_none() {
        let error_msg = "svc-storage unavailable.".to_string();
        rest_error!("(health_check) {}", &error_msg);
        ok = false;
    };

    // let result = grpc_clients.pricing.get_client().await;
    // if result.is_none() {
    //     let error_msg = "svc-pricing unavailable.".to_string();
    //     rest_error!("(health_check) {}", &error_msg);
    //     ok = false;
    // };

    // let result = grpc_clients.scheduler.get_client().await;
    // if result.is_none() {
    //     let error_msg = "svc-scheduler unavailable.".to_string();
    //     rest_error!("(health_check) {}", &error_msg);
    //     ok = false;
    // };

    match ok {
        true => {
            rest_debug!("(health_check) healthy, all dependencies running.");
            Ok(())
        }
        false => {
            rest_error!("(health_check) unhealthy, 1+ dependencies down.");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// Example REST API function
#[utoipa::path(
    post,
    path = "/atc/ack/flight",
    tag = "svc-atc",
    request_body = AckRequest,
    responses(
        (status = 200, description = "Request successful.", body = String),
        (status = 500, description = "Request unsuccessful."),
    )
)]
pub async fn acknowledge_flight_plan(
    Extension(grpc_clients): Extension<GrpcClients>,
    Json(payload): Json<AckRequest>,
) -> Result<(), StatusCode> {
    rest_debug!("(acknowledge_flight_plan) entry.");
    match crate::common::ack_flight(payload.fp_id, &grpc_clients).await {
        Ok(_) => Ok(()),
        Err(e) => {
            rest_error!("(acknowledge_flight_plan) {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
