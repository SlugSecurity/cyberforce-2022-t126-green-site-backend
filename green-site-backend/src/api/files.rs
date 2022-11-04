use actix_web::{
    get, post,
    web::{Path, ServiceConfig},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use log::error;
use serde::Serialize;
use suppaftp::{
    async_native_tls::{Certificate, Protocol, TlsConnector},
    FtpResult, FtpStream, TlsConnector as FtpTlsConnector,
};

use crate::{
    env_vars::BackendVars,
    error::{internal_server_error, ErrorResponse},
};

fn get_tls_connector(cert: &Certificate) -> FtpTlsConnector {
    TlsConnector::new()
        .min_protocol_version(Some(Protocol::Tlsv12))
        .add_root_certificate(cert.clone())
        .use_sni(false)
        .into()
}

async fn secure_ftp_login(vars: &BackendVars, tls_cert: &Certificate) -> FtpResult<FtpStream> {
    let mut ftp_stream = FtpStream::connect((vars.ftps_server_ip.as_str(), vars.ftps_server_port))
        .await?
        .into_secure(get_tls_connector(tls_cert), "domain")
        .await?;

    ftp_stream
        .login(vars.ftps_user.as_str(), vars.ftps_pass.as_str())
        .await?;

    Ok(ftp_stream)
}

fn get_var_and_cert(req: &HttpRequest) -> Option<(&BackendVars, &Certificate)> {
    Some((req.app_data()?, req.app_data()?))
}

macro_rules! verify_var_cert {
    ($req:ident) => {
        match get_var_and_cert(&$req) {
            Some(pair) => pair,
            None => return internal_server_error(),
        }
    };
}

#[derive(Serialize)]
struct File {}

#[get("")]
async fn get_files(req: HttpRequest) -> impl Responder {
    async fn list_files(var: &BackendVars, cert: &Certificate) -> FtpResult<Vec<File>> {
        use suppaftp::list::File as FtpFile;

        let file_list = secure_ftp_login(var, cert).await?.list(None).await?;

        for file in file_list {
            let ftp_file = FtpFile::try_from(file);
        }

        todo!()
    }

    let (var, cert) = verify_var_cert!(req);

    match list_files(var, cert).await {
        Ok(file) => HttpResponse::Ok().json(file),
        Err(err) => {
            error!("Encountered internal error while listing files from FTP server: {err}");

            internal_server_error()
        }
    }
}

#[post("")]
async fn upload_file(req: HttpRequest) -> impl Responder {
    let (var, cert) = verify_var_cert!(req);

    let target_type = mime::MULTIPART_FORM_DATA.essence_str();

    if req.content_type() != target_type {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Content type must be {target_type}."),
        });
    }

    // parse the muultipart form data

    HttpResponse::Ok().body("")
}

#[get("/{file_id}")]
async fn get_file_by_id(req: HttpRequest, path: Path<u64>) -> impl Responder {
    let (var, cert) = verify_var_cert!(req);
    let file_id = path.into_inner();

    HttpResponse::Ok().body("")
}

pub(crate) fn file_endpoint_config(cfg: &mut ServiceConfig) {
    cfg.service(get_files)
        .service(upload_file)
        .service(upload_file);
}
