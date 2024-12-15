use std::convert::Infallible;
use std::fs::File;
use std::net::{IpAddr, SocketAddr};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::header::{HeaderValue, HOST};
use hyper::server::conn as server;
use hyper::service::service_fn;
use hyper::{HeaderMap, Method, Request, Response, StatusCode, Uri};
use hyper_util::rt::TokioIo;
use reqwest::Url;
use tokio::net::TcpListener;

use std::time::{SystemTime, UNIX_EPOCH};

fn current_timestamp_millis() -> u128 {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_epoch.as_millis()
}

fn write_request(
    id: u128,
    uri: &Uri,
    method: &Method,
    headers: &HeaderMap<HeaderValue>,
    body: &Bytes,
) -> Result<(), std::io::Error> {
    use std::io::Write;

    let mut f = File::create(format!("ruxy-request-{}.http", id))?;

    writeln!(f, "{} {} HTTP/1.1", method, uri)?;
    writeln!(f, "X-Ruxy-Created-At: {id}")?;
    for (name, value) in headers {
        writeln!(f, "{}: {}", name, value.to_str().unwrap_or("<unknown>"))?;
    }

    writeln!(f)?;
    f.write_all(body)?;

    Ok(())
}

fn write_response(
    id: u128,
    uri: &str,
    status: &StatusCode,
    headers: &HeaderMap<HeaderValue>,
    body: &Bytes,
) -> Result<(), std::io::Error> {
    use std::io::Write;

    let mut f = File::create(format!("ruxy-response-{}.http", id))?;

    writeln!(
        f,
        "HTTP/1.1 {} {}",
        status.as_u16(),
        status.canonical_reason().unwrap_or_default()
    )?;
    writeln!(f, "X-Ruxy-Created-At: {id}")?;
    writeln!(f, "X-Ruxy-Destination: {uri}")?;
    for (name, value) in headers {
        writeln!(f, "{}: {}", name, value.to_str().unwrap_or("<unknown>"))?;
    }

    writeln!(f)?;
    f.write_all(body)?;

    Ok(())
}

enum RequestError {
    NoHost,
    ForbiddenHost,
    HostUrl,
    Body,
    Request(reqwest::Error),
    ResponseBody,
}

fn send_error(err: RequestError) -> Result<Response<Full<Bytes>>, Infallible> {
    let mut resp = Response::new(Full::new(Bytes::from(match err {
        RequestError::NoHost => "invalid request: no header `Host` passed\nhint: add `Host: domain.com` header to proxy request".to_owned(),
        RequestError::ForbiddenHost => "invalid request: sending request to that host is forbidden\nnote: this error can caused because of you tryed to send request without `Host` header and cURL/browser/etc added it as `localhost`\nhint: try to set `Host` header\nhint: forbidden hosts are localhost and all ipv4/ipv6".into(),
        RequestError::HostUrl => "invalid request: invalid `Host` header passed\nhint: `Host` header is usually normal domain name\nhint: examples are:\n - `example.com`\n - `www.example.com`".to_owned(),
        RequestError::Body => "invalid request: unreadable body\nnote: it may caused because of too big size".to_owned(),
        RequestError::Request(e) => format!("request failed: {e}"),
            RequestError::ResponseBody=>"request failed: server didn't return correct body".to_owned()
    })));

    *resp.status_mut() = StatusCode::BAD_REQUEST;

    Ok(resp)
}

async fn forward(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let id = current_timestamp_millis();

    let headers = req.headers().clone();

    let Some(Ok(host)) = headers.get(HOST).map(HeaderValue::to_str) else {
        return send_error(RequestError::NoHost);
    };

    {
        let host = host
            .split(':')
            .next()
            .expect("at least one element in split");
        if host.trim().is_empty() || host.parse::<IpAddr>().is_ok() || host == "localhost" {
            return send_error(RequestError::ForbiddenHost);
        }
    }

    let uri = req.uri().clone();
    let method = req.method().clone();

    let b = req.into_body();
    let Ok(b) = b.collect().await.map(|v| v.to_bytes()) else {
        return send_error(RequestError::Body);
    };

    if let Err(e) = write_request(id, &uri, &method, &headers, &b) {
        eprintln!("Failed to write request {id}: {e}");
    };

    let Ok(uri) = Url::parse(&format!("https://{}{}", host, uri)) else {
        return send_error(RequestError::HostUrl);
    };

    let response = reqwest::Client::new()
        .request(method, uri.clone())
        .headers(headers)
        .body(b)
        .send()
        .await;

    let response = match response {
        Ok(v) => v,
        Err(e) => return send_error(RequestError::Request(e)),
    };

    let status = response.status();
    let headers = response.headers().clone();
    let Ok(body) = response.bytes().await else {
        return send_error(RequestError::ResponseBody);
    };

    if let Err(e) = write_response(id, uri.as_str(), &status, &headers, &body) {
        eprintln!("Failed to write response {id}: {e}");
    };

    let mut response = Response::new(Full::new(body));
    *response.status_mut() = status;
    *response.headers_mut() = headers;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = server::http1::Builder::new()
                .serve_connection(io, service_fn(forward))
                .await
            {
                eprintln!("Error serving connection: {:?}: {err}", err);
            }
        });
    }
}
