//! Rest API implementations
/// openapi generated rest types
pub mod rest_types {
    include!("../../../openapi/types.rs");
}

pub use rest_types::*;

use crate::grpc::client::GrpcClients;
use axum::{body::Bytes, extract::Extension, Json};
use hyper::StatusCode;

use svc_storage_client_grpc::prelude::*;

// Provides a way to tell a caller if the service is healthy.
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
    Extension(grpc_clients): Extension<GrpcClients>,
) -> Result<(), StatusCode> {
    rest_debug!("(health_check) entry.");

    let mut ok = true;

    // This health check is to verify that ALL dependencies of this
    // microservice are running.
    if grpc_clients
        .storage
        .flight_plan
        .is_ready(ReadyRequest {})
        .await
        .is_err()
    {
        let error_msg = "svc-storage flight_plan unavailable.".to_string();
        rest_error!("(health_check) {}.", &error_msg);
        ok = false;
    }

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

impl TryFrom<flight_plan::Object> for FlightPlan {
    type Error = StatusCode;

    fn try_from(object: flight_plan::Object) -> Result<Self, Self::Error> {
        let flight_uuid = object.id;
        let data = object.data.ok_or_else(|| {
            rest_error!("(try_from) could not get data from object.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let origin_vertiport_id = data.origin_vertiport_id.ok_or_else(|| {
            rest_error!("(try_from) could not get origin_vertiport_id from data.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let target_vertiport_id = data.target_vertiport_id.ok_or_else(|| {
            rest_error!("(try_from) could not get target_vertiport_id from data.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let origin_timeslot_start = data.origin_timeslot_start.ok_or_else(|| {
            rest_error!("(try_from) could not get origin_timeslot_start from data.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let origin_timeslot_end = data.origin_timeslot_end.ok_or_else(|| {
            rest_error!("(try_from) could not get origin_timeslot_end from data.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let target_timeslot_start = data.target_timeslot_start.ok_or_else(|| {
            rest_error!("(try_from) could not get target_timeslot_start from data.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let target_timeslot_end = data.target_timeslot_end.ok_or_else(|| {
            rest_error!("(try_from) could not get target_timeslot_end from data.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let path = data.path.ok_or_else(|| {
            rest_error!("(try_from) could not get path from data.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let path = path
            .points
            .iter()
            .map(|p| PointZ {
                latitude: p.latitude,
                longitude: p.longitude,
                altitude_meters: 100.0, // TODO(R5): Add altitude to path in storage
            })
            .collect();

        Ok(FlightPlan {
            session_id: data.session_id,
            flight_uuid,
            aircraft_id: data.vehicle_id,
            origin_vertiport_id,
            target_vertiport_id,
            origin_vertipad_id: data.origin_vertipad_id,
            target_vertipad_id: data.target_vertipad_id,
            origin_timeslot_start: origin_timeslot_start.into(),
            origin_timeslot_end: origin_timeslot_end.into(),
            target_timeslot_start: target_timeslot_start.into(),
            target_timeslot_end: target_timeslot_end.into(),
            path,
        })
    }
}

/// Acknowledge a flight
#[utoipa::path(
    post,
    path = "/atc/acknowledge",
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

/// Get flight plans
#[utoipa::path(
    get,
    path = "/atc/plans",
    tag = "svc-atc",
    request_body = String,
    responses(
        (status = 200, description = "Request successful.", body = String),
        (status = 500, description = "Request unsuccessful."),
    )
)]
pub async fn get_flight_plans(
    Extension(grpc_clients): Extension<GrpcClients>,
    aircraft_id: Bytes,
) -> Result<Json<Vec<FlightPlan>>, StatusCode> {
    rest_debug!("(get_flight_plans) entry.");
    let aircraft_id = String::from_utf8(aircraft_id.to_vec()).map_err(|_| {
        rest_error!("(get_flight_plans) could not convert aircraft_id to string.");
        StatusCode::BAD_REQUEST
    })?;

    let now = chrono::Utc::now();

    // TODO(R5): parameterize duration lookahead?
    let delta = chrono::Duration::try_minutes(60).ok_or_else(|| {
        rest_error!("(get_flight_plans) could not create duration.");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    //
    // TODO(R5): Move this search to a loop searching for all
    //  aircraft, and push results to local cache
    // So that we can return cached results and not search svc-storage
    //  every time an aircraft asks for flight plans
    //
    let filter = AdvancedSearchFilter::search_equals("vehicle_id".to_owned(), aircraft_id.clone())
        .and_between(
            "origin_timeslot_start".to_owned(),
            (&(now - delta)).to_string(),
            (&(now + delta)).to_string(),
        );

    let client = &grpc_clients.storage.flight_plan;
    let plans = client
        .search(filter)
        .await
        .map_err(|e| {
            let error_msg = "svc-storage failure.".to_string();
            rest_error!("(get_flight_plans) {}: {e}", &error_msg);

            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner()
        .list
        .into_iter()
        .filter_map(|object| FlightPlan::try_from(object).ok())
        .collect::<Vec<FlightPlan>>();

    Ok(Json(plans))
}
