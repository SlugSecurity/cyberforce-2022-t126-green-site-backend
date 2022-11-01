# green-site-backend



## Green Team Website Backend
Contains all of our endpoints for our green team website, which will be documented below.

## Used Environment variables
LDAPS_SERVER_IP
LDAPS_SERVER_PORT
SFTP_SERVER_IP
SFTP_SERVER_PORT
SMTPS_SERVER_IP
SMTPS_SERVER_PORT
DATA_HISTORIAN_IP
DATA_HISTORIAN_PORT
WEB_SERVER_PORT
SSL_CERTIFICATE_PATH
SSL_PRIVATE_KEY_PATH

## Security
Each endpoint's rate limits will be documented. The rate limits shouldn't be hit through regular operation, but should be taken into account on the frontend. If a rate limit is hit, a 429 response code will be given with a Retry-After header in seconds. Each open port listened to will be treated as potentially malicious and will attempt not to process any invalid requests. This should be run in some sort of service that auto-restarts. Communication with the REST API should be done over TLS using the specified self-signed certificate from the environment variables.
