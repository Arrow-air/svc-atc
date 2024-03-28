//! REST
//! provides server implementations for REST API

#[macro_use]
pub mod macros;
pub mod api;
pub mod server;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        api::health_check,
        api::acknowledge_flight_plan,
        api::get_flight_plans,
    ),
    components(
        schemas(
            api::rest_types::AckRequest,
            api::rest_types::AckStatus,
            api::rest_types::PointZ,
            api::rest_types::FlightPlan
        )
    ),
    tags(
        (name = "svc-atc", description = "svc-atc REST API")
    )
)]
struct ApiDoc;

/// Create OpenAPI3 Specification File
pub fn generate_openapi_spec(target: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = ApiDoc::openapi()
        .to_pretty_json()
        .expect("(ERROR) unable to write openapi specification to json.");

    std::fs::write(target, output).expect("(ERROR) unable to write json string to file.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_openapi_spec_generation() {
        crate::get_log_handle().await;
        ut_info!("(test_openapi_spec_generation) Start.");

        assert!(generate_openapi_spec("/tmp/generate_openapi_spec.out").is_ok());

        ut_info!("(test_openapi_spec_generation) Success.");
    }
}
