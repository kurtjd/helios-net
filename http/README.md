# :sunny: helios-http
Just a simple web server.

# Features
- Loosely "supports" HTTP/1.0 and HTTP/1.1
- Supports TLS/HTTPS
- Supports PHP CGI
- Configurable via text file

# Usage
`helios-http <path-to-config-file>`

If no config file is passed, will use default settings.

# Config
Uses a text file to configure server.
An example config file (with all settings set to default) looks like:

```
server_root=/var/www
max_connections=10
max_header_len=8192
max_body_len=1048576
max_timeout=5
ip=127.0.0.1
port_http=1337
port_https=31337
https_enabled=true
```

Within the server root folder, the server expects several additional folders:
- `public`: Contains all publicly accessible web pages and files.
- `errors`: Used for custom error pages, with an error number mapping as a filename (e.g. `404.html`).
- `crypt`: If HTTPS enabled, must contain files `public.pem` and `private.pem` (public and private keys respectively).