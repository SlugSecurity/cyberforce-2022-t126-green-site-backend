use actix_web::{get, web::ServiceConfig, HttpRequest, HttpResponse, Responder};
use log::error;
use serde::Serialize;
use sqlx::{FromRow, MySqlPool};

use crate::env_vars::BackendVars;
use crate::error::{self, MISSING_APP_DATA};

#[derive(FromRow, Serialize)]
struct SolarPanelInfo {
    #[sqlx(rename = "arrayID")]
    array_id: i32,
    #[sqlx(rename = "solarStatus")]
    solar_status: String,
    #[sqlx(rename = "arrayVoltage")]
    array_voltage: i32,
    #[sqlx(rename = "arrayCurrent")]
    array_current: i32,
    #[sqlx(rename = "arrayTemp")]
    array_temp: i32,
    #[sqlx(rename = "trackerTilt")]
    tracker_tilt: i32,
    #[sqlx(rename = "trackerAzimuth")]
    tracker_azimuth: i32,
}

async fn get_solar_panel_info(
    pool: &MySqlPool,
    vars: &BackendVars,
) -> sqlx::Result<Vec<SolarPanelInfo>> {
    sqlx::query_as(
        format!(
            "SELECT arrayID, solarStatus, \
             arrayVoltage, arrayCurrent, arrayTemp, \
             trackerTilt, trackerAzimuth \
         FROM {};",
            vars.data_historian_db_table.as_str(),
        )
        .as_str(),
    )
    .persistent(true)
    .fetch_all(pool)
    .await
}

#[get("")]
async fn get_solar_data(req: HttpRequest) -> impl Responder {
    if let (Some(pool), Some(vars)) = (req.app_data::<MySqlPool>(), req.app_data::<BackendVars>()) {
        match get_solar_panel_info(pool, vars).await {
            Ok(info) => HttpResponse::Ok().json(info),
            Err(err) => {
                error!("Encountered sqlx error getting solar panel info: {err}");

                error::internal_server_error()
            }
        }
    } else {
        error!(
            "{MISSING_APP_DATA}. BackendVars: {:?}; MySQL Connection Pool: {:?}",
            req.app_data::<BackendVars>(),
            req.app_data::<MySqlPool>()
        );

        error::internal_server_error()
    }
}

pub(crate) fn solar_endpoint_config(cfg: &mut ServiceConfig) {
    cfg.service(get_solar_data);
}
