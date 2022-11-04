use green_site_backend_macros::env_vars;

#[env_vars]
#[env_var("LDAPS_SERVER_IP", String)]
#[env_var("LDAPS_SERVER_PORT", u16)]
#[env_var("FTPS_SERVER_IP", String)]
#[env_var("FTPS_SERVER_PORT", u16)]
#[env_var("FTPS_USER", String)]
#[env_var("FTPS_PASS", String)]
#[env_var("EMAIL_SERVER_IP", String)]
#[env_var("SMTP_SERVER_PORT", u16)]
#[env_var("IMAP_SERVER_PORT", u16)]
#[env_var("EMAIL_USER", String)]
#[env_var("EMAIL_PASS", String)]
#[env_var("DATA_HISTORIAN_IP", String)]
#[env_var("DATA_HISTORIAN_PORT", u16)]
#[env_var("DATA_HISTORIAN_USER", String)]
#[env_var("DATA_HISTORIAN_PASS", String)]
#[env_var("DATA_HISTORIAN_DB_NAME", String)]
#[env_var("DATA_HISTORIAN_DB_TABLE", String)]
#[env_var("WEB_SERVER_PORT", u16)]
#[env_var("DATA_SUBMISSION_LIMIT", usize)]
#[env_var("DATA_SUBMISSION_RATE_LIMIT", usize)]
#[env_var("DEFAULT_RATE_LIMIT", usize)]
#[env_var("ADMIN_ACCOUNT_USERNAME", String)]
#[env_var("ADMIN_TOKEN", String)]
#[env_var("SSL_CERTIFICATE_PEM_PATH", String)]
#[env_var("SSL_PRIVATE_KEY_PEM_PATH", String)]
#[env_var("ROOT_CERTIFICATE_PATH", String)]
pub(crate) struct BackendVars;
