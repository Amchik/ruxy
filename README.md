# ruxy

> idk if this name taken

> tldr: Yet another local shitty script

Ruxy is a simple reverse proxy with request logging.
I use it with nginx.

###### Examples

```console
$ setsid -f ./ruxy
$ curl -H "Host: httpbin.org" localhost:3000/get
$ cat ruxy-response[...].http
HTTP/1.1 200 OK
[...]
```

###### Problems

Runs only on `0.0.0.0:3000` but i don't care

