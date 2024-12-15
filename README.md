# ruxy

> idk if this name taken

> tldr: Yet another local shitty script

Ruxy is a simple reverse proxy with request logging.
I use it with nginx.

###### Examples

```console
$ ./ruxy --help
Simple reverse proxy with request logging.

Usage: ruxy [OPTIONS]

Options:
      --ip <IP>      IP address to listen [default: 127.0.0.1]
  -p, --port <PORT>  Port to listen [default: 3000]
  -h, --help         Print help
```

```console
$ setsid -f ./ruxy
$ curl -H "Host: httpbin.org" localhost:3000/get
$ cat ruxy-response[...].http
HTTP/1.1 200 OK
[...]
```

###### Example nginx config

```nginx
worker_processes auto;
pid /tmp/nginx.pid;
error_log /tmp/nginx-error.log;

events {
    worker_connections 1024;
}

http {
    ssl_protocols TLSv1.2;
    access_log /tmp/nginx-access.log;

    # <<< contents of /etc/nginx/sites-enables/...
    server {
        listen 80;
        listen 443 ssl;

        # for fake certs
        server_name         dns.google;
        ssl_certificate     dns.google.crt;
        ssl_certificate_key dns.google.key;

        location / {
            # change port if needed
            proxy_pass http://127.0.0.1:3000/$request_uri;
        }
    }
    # >>> end contents of /etc/nginx/sites-enables/...
}

# TIP: this config can be runned as unprivileged user (change `listen` ports)
```

