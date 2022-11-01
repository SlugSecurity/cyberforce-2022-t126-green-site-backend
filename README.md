# green-site-backend



## Green Team Website Backend
Contains all of our endpoints for our green team website, which will be documented below.

## Used Environment variables
- LDAPS_SERVER_IP - IP of LDAP/S server
- LDAPS_SERVER_PORT - Port of LDAP/S server
- FTPS_SERVER_IP - IP of FTP/S server
- FTPS_SERVER_PORT - Port of FTP/S server
- SMTPS_SERVER_IP - IP of SMTP/S server
- SMTPS_SERVER_PORT - IP of SMTP/S server
- DATA_HISTORIAN_IP - IP of Data Historian Database
- DATA_HISTORIAN_PORT - Port of Data Historian Database
- WEB_SERVER_PORT - Port of Web Server
- SSL_CERTIFICATE_PEM_PATH - Path of SSL certificate PEM
- SSL_PRIVATE_KEY_PEM_PATH - Path of Private key PEM
- FILE_UPLOAD_LIMIT - The limit in bytes of the size of file uploads done
- TEXT_FORM_LIMIT - The limit in bytes of the size of each individual text form (username, name, email, phone number) submitted
- PASSWORD_LIMIT - The limit in bytes of the size of entered passwords
- FORM_SUBMISSION_RATE_LIMIT - The number of requests allowed per second for login and contact form submissions.
- DEFAULT_RATE_LIMIT - The number of requests allowed per second for all other applicable endpoints that don't have a custom rate limit.

## Security
Each endpoint's rate limits will be documented. The rate limits shouldn't be hit through regular operation, but should be taken into account on the frontend. If a rate limit is hit, a 429 response code will be given with a Retry-After header in seconds. Each open port listened to will be treated as potentially malicious and will attempt not to process any invalid requests. This should be run in some sort of service that auto-restarts. Communication with the REST API should be done over TLS using the specified self-signed certificate from the environment variables.


## Security Checklist (to review after finished)
- [x] Completed
- [~] Inapplicable
- [ ] Incomplete
-----------------------------
- [ ] Ensure file upload byte limit is enforced.
- [ ] Ensure name byte limit in contact form is enforced.
- [ ] Ensure email byte limit in contact form is enforced.
- [ ] Ensure phone number byte limit in contact form is enforced.
- [ ] Ensure username byte limit is enforced.
- [ ] Ensure password byte limit is enforced.
- [ ] Ensure custom rate limit for form submission is enforced.
- [ ] Ensure custom rate limit for login submission is enforced.
- [ ] Ensure default rate limit is enforced for all other applicable endpoints.
- [ ] Ensure TLS is being used for SMTP.
- [ ] Ensure TLS is being used for LDAP.
- [ ] Ensure TLS is being used for FTP.
- [ ] Ensure TLS is being used for MySQL if possible?
- [ ] Ensure TLS is being used for frontend communication with self-signed cert if possible?
- [ ] Ensure garbage inputs on SMTP connection doesn't crash.
- [ ] Ensure garbage inputs on LDAP connection doesn't crash.
- [ ] Ensure garbage inputs on FTP connection doesn't crash.
- [ ] Ensure garbage inputs on MySQL connection doesn't crash.
- [ ] Ensure garbage inputs from frontend connection doesn't crash.
