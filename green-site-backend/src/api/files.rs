use std::{error::Error, fmt::Display};

use actix_multipart::{Multipart, MultipartError};
use actix_web::{
    get, post,
    web::{Path, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};

use futures::{AsyncRead, AsyncReadExt, StreamExt};
use log::error;
use rand::Rng;
use serde::Serialize;
use suppaftp::{
    async_native_tls::{Certificate, Protocol, TlsConnector},
    types::Response,
    FtpError, FtpResult, FtpStream, Status, TlsConnector as FtpTlsConnector,
};

use crate::{
    env_vars::BackendVars,
    error::{internal_server_error, ErrorResponse},
    verify_admin_token,
};

fn get_var_and_ftp_cert(req: &HttpRequest) -> Option<(&BackendVars, &Certificate)> {
    Some((req.app_data()?, req.app_data()?))
}

macro_rules! verify_var_cert {
    ($req:ident) => {
        match get_var_and_ftp_cert(&$req) {
            Some(pair) => pair,
            None => return crate::error::internal_server_error(),
        }
    };
}

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
        .into_secure(get_tls_connector(tls_cert), &vars.ftps_server_ip)
        .await?;

    ftp_stream
        .login(vars.ftps_user.as_str(), vars.ftps_pass.as_str())
        .await?;

    Ok(ftp_stream)
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

    verify_admin_token!(req, var);

    match list_files(var, cert).await {
        Ok(files) => HttpResponse::Ok().json(files),
        Err(err) => {
            error!("Encountered internal error while listing files from FTP server: {err}");

            internal_server_error()
        }
    }
}

#[derive(Debug)]
enum UploadError {
    UpFtpError(FtpError),
    UpMultipartError(MultipartError),
    NoData,
    BadFileName,
}

impl Display for UploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadError::UpFtpError(err) => write!(f, "{err}"),
            UploadError::UpMultipartError(err) => write!(f, "{err}"),
            UploadError::NoData => write!(f, "No data in multipart"),
            UploadError::BadFileName => write!(f, "Bad file name multipart"),
        }
    }
}

impl Error for UploadError {}

impl From<FtpError> for UploadError {
    fn from(value: FtpError) -> Self {
        UploadError::UpFtpError(value)
    }
}

impl From<MultipartError> for UploadError {
    fn from(value: MultipartError) -> Self {
        UploadError::UpMultipartError(value)
    }
}

#[post("")]
async fn upload_file(req: HttpRequest, multi_part: Multipart) -> impl Responder {
    use UploadError::*;

    async fn ftp_upload(
        vars: &BackendVars,
        cert: &Certificate,
        mut file: Multipart,
    ) -> Result<(), UploadError> {
        let mut field = file.next().await.ok_or(NoData)??;
        let mut bytes_vec = Vec::new();

        loop {
            if let Some(bytes_res) = field.next().await {
                bytes_vec.extend(bytes_res?);
            } else {
                break;
            }
        }

        if bytes_vec.is_empty() {
            return Err(NoData);
        }

        let mut conn = secure_ftp_login(vars, cert).await?;
        let rand_id = rand::thread_rng().gen::<u128>();
        let file_name = field
            .content_disposition()
            .get_filename()
            .ok_or(UploadError::BadFileName)?;

        conn.put_file(format!("{rand_id}-{file_name}"), &mut &bytes_vec[..])
            .await?;

        Ok(())
    }

    let (var, cert) = verify_var_cert!(req);

    match ftp_upload(var, cert, multi_part).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(UpFtpError(FtpError::UnexpectedResponse(Response {
            status: Status::BadFilename,
            ..
        }))) => HttpResponse::BadRequest().json(ErrorResponse {
            error: "Bad file name".to_string(),
        }),
        Err(UpMultipartError(_) | NoData | BadFileName) => {
            HttpResponse::BadRequest().json(ErrorResponse {
                error: "Malformed multipart file".to_string(),
            })
        }
        Err(UpFtpError(err)) => {
            error!("Encountered FTP error while uploading file: {err}");

            internal_server_error()
        }
    }
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

    verify_admin_token!(req, var);

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
            if let Err(err) = data.read_to_end(&mut data_vec).await {
                error!("Encountered IO error downloading file: {err}");

                return internal_server_error();
            }

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
