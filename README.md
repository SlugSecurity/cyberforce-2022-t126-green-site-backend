# green-site-backend



## Green Team Website Backend
Contains all of our endpoints for our green team website, which will be documented below.

## Used Environment variables
LDAPS_SERVER_IP - IP of LDAP/S server
LDAPS_SERVER_PORT - Port of LDAP/S server
FTPS_SERVER_IP - IP of FTP/S server
FTPS_SERVER_PORT - Port of FTP/S server
SMTPS_SERVER_IP - IP of SMTP/S server
SMTPS_SERVER_PORT - IP of SMTP/S server
DATA_HISTORIAN_IP - IP of Data Historian Database
DATA_HISTORIAN_PORT - Port of Data Historian Database
WEB_SERVER_PORT - Port of Web Server
SSL_CERTIFICATE_PEM_PATH - Path of SSL certificate PEM
SSL_PRIVATE_KEY_PEM_PATH - Path of Private key PEM

## Security
Each endpoint's rate limits will be documented. The rate limits shouldn't be hit through regular operation, but should be taken into account on the frontend. If a rate limit is hit, a 429 response code will be given with a Retry-After header in seconds. Each open port listened to will be treated as potentially malicious and will attempt not to process any invalid requests. This should be run in some sort of service that auto-restarts. Communication with the REST API should be done over TLS using the specified self-signed certificate from the environment variables.
