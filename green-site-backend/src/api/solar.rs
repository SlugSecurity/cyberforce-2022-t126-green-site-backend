use actix_web::{get, web::ServiceConfig, HttpRequest, HttpResponse, Responder};
use log::error;
use serde::Serialize;
use sqlx::{FromRow, MySqlPool};

use crate::env_vars::BackendVars;
use crate::error::{self, MISSING_APP_DATA};

#[derive(FromRow, Serialize)]
#[sqlx(rename_all = "camelCase")]
struct SolarPanelInfo {
    array_id: i32,
    solar_status: String,
    array_voltage: i32,
    array_current: i32,
    array_temp: i32,
    tracker_tilt: i32,
    tracker_azimuth: i32,
}

async fn get_solar_panel_info(
    pool: &MySqlPool,
    vars: &BackendVars,
) -> sqlx::Result<Vec<SolarPanelInfo>> {
    sqlx::query_as(
        "SELECT arrayID, solarStatus, \
             arrayVoltage, arrayCurrent, arrayTemp, \
             trackerTilt, trackerAzimuth \
         FROM ?",
    )
    .bind(vars.data_historian_db_table.as_str())
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
