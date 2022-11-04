use actix_web::{
    get, post,
    web::{Path, ServiceConfig},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};

use async_std::io::ReadExt;
use bytes::Bytes;
use futures::{
    io::AsyncBufReadExt,
    stream::{self, once},
    AsyncRead, Stream, StreamExt,
};
use log::error;
use serde::Serialize;
use suppaftp::{
    async_native_tls::{Certificate, Protocol, TlsConnector},
    types::Response,
    FtpError, FtpResult, FtpStream, Status, TlsConnector as FtpTlsConnector,
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
struct File {
    name: String,
    id: String,
    size: u64,
}

fn split_name(name: &str) -> Option<(String, String)> {
    let split = name
        .find('-')
        .filter(|&idx| name[..idx].bytes().all(|c| c.is_ascii_digit()) && name.len() > idx + 1)?;

    Some((name[..split].to_string(), name[split + 1..].to_string()))
}

async fn list_files(var: &BackendVars, cert: &Certificate) -> FtpResult<Vec<File>> {
    use suppaftp::list::File as FtpFile;

    let file_list = secure_ftp_login(var, cert).await?.list(None).await?;
    let mut processed_files = Vec::new();

    for file in file_list {
        let ftp_file = FtpFile::try_from(file).map_err(|_| FtpError::BadResponse)?;
        let (name, id) = match split_name(ftp_file.name()) {
            Some(pair) => pair,
            None => return Err(FtpError::BadResponse),
        };
        let processed_file = File {
            name,
            id,
            size: ftp_file.size() as u64,
        };

        processed_files.push(processed_file)
    }

    Ok(processed_files)
}

#[get("")]
async fn get_files(req: HttpRequest) -> impl Responder {
    let (var, cert) = verify_var_cert!(req);

    match list_files(var, cert).await {
        Ok(files) => HttpResponse::Ok().json(files),
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
async fn get_file_by_id(req: HttpRequest, path: Path<u128>) -> impl Responder {
    async fn download_file(
        vars: &BackendVars,
        tls_cert: &Certificate,
        name: String,
    ) -> FtpResult<(impl AsyncRead, FtpStream)> {
        let mut conn = secure_ftp_login(vars, tls_cert).await?;

        Ok((conn.retr_as_stream(name).await?, conn))
    }

    let (var, cert) = verify_var_cert!(req);
    let file_id = path.to_string();

    let files = match list_files(var, cert).await {
        Ok(files) => files,
        Err(err) => {
            error!("Encountered internal error while listing files from FTP server: {err}");

            return internal_server_error();
        }
    };
    let found_file = match files.into_iter().find(|f| f.id == file_id) {
        Some(file) => file,
        None => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Couldn't find requested file by ID".to_string(),
            })
        }
    };

    match download_file(var, cert, format!("{}-{}", found_file.id, found_file.name)).await {
        Ok((mut data, mut stream)) => {
            let mut data_vec = Vec::new();
            data.read_to_end(&mut data_vec);

            if let Err(err) = stream.finalize_retr_stream(data).await {
                error!("We couldn't finalize the FTP stream: {err}");

                internal_server_error()
            } else {
                HttpResponse::Ok().body(data_vec)
            }
        }
        Err(err) => match err {
            FtpError::UnexpectedResponse(Response {
                status: Status::BadFilename,
                ..
            }) => HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("File with requested ID doesn't exist."),
            }),
            err => {
                error!("We had an internal error while trying to get our FTP file: {err}");

                internal_server_error()
            }
        },
    }
}

pub(crate) fn file_endpoint_config(cfg: &mut ServiceConfig) {
    cfg.service(get_files)
        .service(upload_file)
        .service(upload_file);
}
