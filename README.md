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

