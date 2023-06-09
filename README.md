# Green Site Backend

This repository contains the Green Team site backend created by UCSC's Cyber Slugs (Team 126) for the Department of Energy's 2022 CyberForce Competition.

Team Members: Jeffrey Zhang, Brian Mak, Steven Mak, Jackson Kohls, Nancy Lau

Note: This does not represent our best work as we only had a very limited amount of time to prepare the website.

## Description
Contains all of the endpoints for our Green Team website, which are documented below.

## Used Environment variables
- SQLITE_FILE_NAME - Name of SQLite DB
- FTPS_SERVER_IP - IP of FTPS server
- FTPS_SERVER_PORT - Port of FTPS server
- FTPS_USER - The username to log into the FTPS server
- FTPS_PASS - The password to log into the FTPS server
- EMAIL_SERVER_IP - IP of mail server (Needs SMTP and IMAP STARTTLS support)
- SMTP_SERVER_PORT - Port of SMTP server
- IMAP_SERVER_PORT - IP of IMAP server
- EMAIL_USER - The username to log into the mail server
- EMAIL_PASS - The password to log into the mail server
- DATA_HISTORIAN_IP - IP of Data Historian database
- DATA_HISTORIAN_PORT - Port of Data Historian database
- DATA_HISTORIAN_USER - The username to log into the Data Historian database
- DATA_HISTORIAN_PASS - The password to log into the Data Historian database
- DATA_HISTORIAN_DB_NAME - The name of the database that contains the solar panel array info.
- DATA_HISTORIAN_DB_TABLE - The database table that contains the solar panel array info.
- WEB_SERVER_PORT - Port of web server backend
- ADMIN_ACCOUNT_USERNAME - The username of the admin.
- ADMIN_TOKEN - A string of characters to use as the token to send to admins.
- SSL_CERTIFICATE_PEM_PATH - Path of SSL certificate PEM
- SSL_PRIVATE_KEY_PEM_PATH - Path of private key PEM
- ROOT_CERTIFICATE_PATH - Path of root certificate

## Endpoint Documentation (See next section down for object documentation.)
Any JSON data sent via a POST request should have a content type of 'application/json' unless it's a file upload in which case 'multipart/form-data' should be used. 

Authentication is token-based that's returned when logging in. Privileged endpoints as specified below can only be accessed by admin accounts using the token in the Authorization header with type ``Bearer``. 

If a provided endpoint's service is down, response code 503 will be given.

Any 40x and 50x response codes returned will also return an object containing one ``error`` field which is a string with the error message.

- /api/login - POST request endpoint. The request body should be a ``UserLogin`` object. Responds with an ``Authentication`` object.
  - Response code 400 if UserLogin is malformed, or username isn't all lowercase ASCII characters.
  - Response code 401 if credentials are invalid.
- /api/solar - GET request endpoint to retrieve solar panel info. Responds with a ``[SolarPanelInfo]`` object.
- /api/files - Privileged GET request endpoint to retrieve all file metadata from the FTP server. Returns [File].
  - Response code 401 if authorization token is invalid.
- /api/files - POST request endpoint to upload a file to the FTP server. This should be a ``multipart/form-data`` where the content disposition header has ``form-data`` as the first directive followed by the ``filename`` directive that is between 1-72 characters.
  - Response code 400 if content type isn't multipart/form-data with valid form data, filename directive isn't provided, or file name isn't set to a valid file name between 1 and 72 characters.
- /api/files/**ID** - Privileged GET request endpoint to download a file from the FTP server by ID. Returns the file data in the response body with the content type set to 'application/octet-stream' and content disposition set to ``attachment; filename="<FILE_NAME>"``.
  - Response code 400 if file with provided ID doesn't exist.
  - Response code 401 if authorization token is invalid.
- /api/emails - Privileged GET request endpoint to get all stored emails. Returns ``[Email]`` on success.
  - Response code 401 if authorization token is invalid.
- /api/emails - POST request endpoint to send an email. The request body should be an ``Email`` object.
  - Response code 400 if Email is malformed.

## Object documentation
```
UserLogin {
    username: string (1 char min, 72 char limit, all lowercase characters),
    password: string (1 char min, 72 char limit)
}
```
```
Authentication {
    token: string? (only sent if the user is an admin)
}
```
```
SolarPanelInfo {
    array_id: number (32 bits signed),
    solar_status: string, (do not turn into a number)
    array_voltage: number (32 bits signed),
    array_current: number (32 bits signed),
    array_temp: number (32 bits signed),
    tracker_tilt: number (32 bits signed),
    tracker_azimuth: number (32 bits signed)
}
```
```
File {
    name: string (72 char max)
    id: string
    size: number (64 bit unsigned)
}
```
```
Email {
    subject: string (1 char min, 100 char max)
    from_name: string (1 char min, 100 char max)
    from_email: string(1 char min, 100 char max)
    body: string
}
```

## Security
Each open port listened to will be treated as potentially malicious and will attempt not to process any invalid requests. This should be run in some sort of service that auto-restarts. Communication with the REST API should be done over TLS using the specified self-signed certificate from the environment variables.


## Security Checklist (to review after finished)
- [x] Completed
- [~] Inapplicable
- [ ] Incomplete
-----------------------------
- [ ] Working directory of web server application is only accessible by web server user and root.
- [ ] Ensure file upload names are sanitized.
- [ ] Ensure file upload byte limit is enforced.
- [ ] Ensure size limit for form submission is enforced.
- [ ] Ensure size limits for login submission is enforced.
- [ ] Ensure custom rate limit for form submission is enforced.
- [ ] Ensure custom rate limit for login submission is enforced.
- [ ] Ensure default rate limit is enforced for all other applicable endpoints.
- [ ] Ensure TLS is being used for SMTP and only allows secure ciphersuites.
- [ ] Ensure TLS is being used for IMAP and only allows secure ciphersuites.
- [ ] Ensure TLS is being used for LDAP and only allows secure ciphersuites.
- [ ] Ensure TLS is being used for FTP and only allows secure ciphersuites.
- [ ] Ensure TLS is being used for MySQL if possible and only allows secure ciphersuites?
- [ ] Ensure TLS is being used for frontend communication with self-signed cert if possible and only allows secure ciphersuites?
- [ ] Ensure garbage inputs on SMTP connection doesn't crash/hang.
- [ ] Ensure garbage inputs on IMAP connection doesn't crash/hang.
- [ ] Ensure garbage inputs on LDAP connection doesn't crash/hang.
- [ ] Ensure garbage inputs on FTP connection doesn't crash/hang.
- [ ] Ensure garbage inputs on MySQL connection doesn't crash/hang.
- [ ] Ensure garbage inputs from frontend connection doesn't crash/hang.
- [ ] Log in as all users to pregenerate tokens.
