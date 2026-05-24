use crate::log;
use crate::Config;
use bytes::Bytes;
use futures_util::TryStreamExt;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use hyper::{Method, Request, Response, StatusCode};
use mime_guess::from_path;
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use std::convert::Infallible;
use std::path::{Component, Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;

type Resp = Response<BoxBody<Bytes, std::io::Error>>;

const MAX_PREVIEW_SIZE: u64 = 1024 * 1024 * 4;

pub async fn handle(
    req: Request<Incoming>,
    cfg: Config,
) -> Result<Resp, Infallible> {
    let uri_path = req.uri().path().to_string();

    let query = req.uri().query().unwrap_or("");

    let path = sanitize(&uri_path, &cfg.root);

    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("local");

    let method = req.method().to_string();

    let res = match *req.method() {
        Method::GET | Method::HEAD => {
            route(path, &uri_path, query, &cfg, &method, ip).await
        }
        _ => {
            log::request(&method, &uri_path, 405, ip);
            plain(StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed")
        }
    };

    Ok(res)
}

async fn route(
    path: PathBuf,
    uri: &str,
    query: &str,
    cfg: &Config,
    method: &str,
    ip: &str,
) -> Resp {
    if !path.exists() {
        log::request(method, uri, 404, ip);

        return plain(StatusCode::NOT_FOUND, "404 Not Found");
    }

    if path.is_dir() {
        let index = path.join("index.html");

        if index.exists() && !cfg.ignore_index {
            return raw_file(index, uri, method, ip, false).await;
        }

        return listing(path, uri, method, ip).await;
    }

    if query.contains("download=1") {
        return raw_file(path, uri, method, ip, true).await;
    }

    preview_file(path, uri, method, ip).await
}

async fn preview_file(
    path: PathBuf,
    uri: &str,
    method: &str,
    ip: &str,
) -> Resp {
    let meta = match fs::metadata(&path).await {
        Ok(m) => m,
        Err(_) => {
            log::request(method, uri, 500, ip);
            return plain(StatusCode::INTERNAL_SERVER_ERROR, "500");
        }
    };

    let size = meta.len();

    let ext = path
        .extension()
        .map(|v| v.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let previewable = matches!(
        ext.as_str(),
        "txt"
            | "md"
            | "rs"
            | "js"
            | "ts"
            | "json"
            | "html"
            | "css"
            | "toml"
            | "yaml"
            | "yml"
            | "py"
            | "java"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "go"
            | "sh"
            | "php"
            | "xml"
            | "swift"
            | "kt"
            | "lua"
            | "sql"
            | "rb"
    );

    if !previewable || size > MAX_PREVIEW_SIZE {
        return raw_file(path, uri, method, ip, false).await;
    }

    let mut file = match fs::File::open(&path).await {
        Ok(f) => f,
        Err(_) => {
            log::request(method, uri, 500, ip);
            return plain(StatusCode::INTERNAL_SERVER_ERROR, "500");
        }
    };

    let mut buf = String::new();

    if file.read_to_string(&mut buf).await.is_err() {
        return raw_file(path, uri, method, ip, false).await;
    }

    let name = path
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());

    let escaped = html_escape(&buf);

    let download = format!("{}?download=1", uri);

    let html = format!(
        r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{}</title>
<style>
*{{box-sizing:border-box}}
body{{margin:0;background:#020617;color:#e2e8f0;font-family:Inter,Arial,sans-serif}}
.top{{position:sticky;top:0;background:#0f172a;padding:18px;border-bottom:1px solid #1e293b;display:flex;justify-content:space-between;align-items:center}}
.left{{display:flex;gap:12px;align-items:center}}
a{{text-decoration:none}}
.btn{{background:#2563eb;color:white;padding:10px 16px;border-radius:12px;font-size:14px}}
.btn:hover{{background:#1d4ed8}}
.wrap{{padding:20px}}
pre{{margin:0;background:#0f172a;border:1px solid #1e293b;border-radius:18px;padding:24px;overflow:auto;line-height:1.5}}
.meta{{color:#94a3b8;font-size:14px}}
</style>
</head>
<body>
<div class="top">
<div class="left">
<a class="btn" href="javascript:history.back()">← Back</a>
<div>
<div>{}</div>
<div class="meta">{} bytes</div>
</div>
</div>
<a class="btn" href="{}">Download</a>
</div>
<div class="wrap">
<pre>{}</pre>
</div>
</body>
</html>"#,
        name,
        name,
        size,
        download,
        escaped
    );

    log::served(&path.display().to_string(), size);
    log::request(method, uri, 200, ip);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(
            Full::new(Bytes::from(html))
                .map_err(|never| match never {})
                .boxed(),
        )
        .unwrap()
}

async fn raw_file(
    path: PathBuf,
    uri: &str,
    method: &str,
    ip: &str,
    attachment: bool,
) -> Resp {
    match fs::File::open(&path).await {
        Ok(file) => {
            let meta = file.metadata().await.ok();
            let size = meta.map(|m| m.len()).unwrap_or(0);

            let mime = from_path(&path).first_or_octet_stream();

            let stream = ReaderStream::new(file)
                .map_ok(Frame::data);

            let body = StreamBody::new(stream).boxed();

            log::served(&path.display().to_string(), size);
            log::request(method, uri, 200, ip);

            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, mime.as_ref());

            if attachment {
                let name = path
                    .file_name()
                    .map(|v| v.to_string_lossy().to_string())
                    .unwrap_or_else(|| "file".to_string());

                builder = builder.header(
                    CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", name),
                );
            }

            builder.body(body).unwrap()
        }
        Err(_) => {
            log::request(method, uri, 500, ip);
            plain(StatusCode::INTERNAL_SERVER_ERROR, "500")
        }
    }
}

async fn listing(
    path: PathBuf,
    uri: &str,
    method: &str,
    ip: &str,
) -> Resp {
    let mut html = String::new();

    html.push_str("<!doctype html><html><head><meta charset=utf-8>");
    html.push_str("<meta name=viewport content=\"width=device-width,initial-scale=1\">");
    html.push_str("<title>serve</title>");

    html.push_str("<style>");
    html.push_str("*{box-sizing:border-box}");
    html.push_str("body{margin:0;background:#020617;color:#e2e8f0;font-family:Inter,Arial,sans-serif}");
    html.push_str(".top{position:sticky;top:0;padding:18px;background:#0f172a;border-bottom:1px solid #1e293b;z-index:100}");
    html.push_str(".crumbs{display:flex;flex-wrap:wrap;gap:10px}");
    html.push_str(".crumbs a{background:#082f49;color:#38bdf8;padding:8px 14px;border-radius:12px;text-decoration:none;font-size:14px}");
    html.push_str(".crumbs a:hover{background:#0c4a6e}");
    html.push_str(".wrap{padding:22px}");
    html.push_str(".grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(260px,1fr));gap:14px}");
    html.push_str(".card{display:flex;justify-content:space-between;align-items:center;padding:18px;border-radius:18px;background:#0f172a;border:1px solid #1e293b;text-decoration:none;color:#fff;transition:.15s}");
    html.push_str(".card:hover{transform:translateY(-2px);border-color:#38bdf8;background:#111827}");
    html.push_str(".left{display:flex;gap:12px;align-items:center;overflow:hidden}");
    html.push_str(".icon{font-size:20px}");
    html.push_str(".name{text-overflow:ellipsis;overflow:hidden;white-space:nowrap}");
    html.push_str(".tag{font-size:11px;padding:6px 10px;border-radius:999px}");
    html.push_str(".dir{background:#2563eb}");
    html.push_str(".file{background:#475569}");
    html.push_str("</style></head><body>");

    html.push_str("<div class=top><div class=crumbs>");

    html.push_str("<a href=\"/\">root</a>");

    let mut current = String::new();

    for comp in Path::new(uri).components() {
        if let Component::Normal(seg) = comp {
            let seg = seg.to_string_lossy();

            current.push('/');

            current.push_str(
                &utf8_percent_encode(
                    &seg,
                    NON_ALPHANUMERIC
                )
                .to_string(),
            );

            html.push_str(&format!(
                "<a href=\"{}\">{}</a>",
                current,
                seg
            ));
        }
    }

    html.push_str("</div></div>");

    html.push_str("<div class=wrap>");
    html.push_str("<div class=grid>");

    let mut rd = fs::read_dir(path).await.unwrap();

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    while let Ok(Some(entry)) = rd.next_entry().await {
        let ty = entry.file_type().await.unwrap();
        let name = entry.file_name().to_string_lossy().to_string();

        if ty.is_dir() {
            dirs.push(name);
        } else {
            files.push(name);
        }
    }

    dirs.sort();
    files.sort();

    let dir_count = dirs.len();
    let file_count = files.len();

    if uri != "/" {
        let trimmed = uri.trim_end_matches('/');

        let parent = trimmed
            .rsplit_once('/')
            .map(|v| {
                if v.0.is_empty() {
                    "/".to_string()
                } else {
                    v.0.to_string()
                }
            })
            .unwrap_or_else(|| "/".to_string());

        html.push_str(&format!(
            "<a class=card href=\"{}\"><div class=left><span class=icon>⬅️</span><span class=name>..</span></div><span class=\"tag dir\">UP</span></a>",
            parent
        ));
    }

    for name in dirs {
        let encoded = utf8_percent_encode(
            &name,
            NON_ALPHANUMERIC
        )
        .to_string();

        let href = if uri == "/" {
            format!("/{}", encoded)
        } else {
            format!("{}/{}", uri.trim_end_matches('/'), encoded)
        };

        html.push_str(&format!(
            "<a class=card href=\"{}\"><div class=left><span class=icon>📁</span><span class=name>{}</span></div><span class=\"tag dir\">DIR</span></a>",
            href,
            name
        ));
    }

    for name in files {
        let encoded = utf8_percent_encode(
            &name,
            NON_ALPHANUMERIC
        )
        .to_string();

        let href = if uri == "/" {
            format!("/{}", encoded)
        } else {
            format!("{}/{}", uri.trim_end_matches('/'), encoded)
        };

        html.push_str(&format!(
            "<a class=card href=\"{}\"><div class=left><span class=icon>📄</span><span class=name>{}</span></div><span class=\"tag file\">FILE</span></a>",
            href,
            name
        ));
    }

    html.push_str("</div></div></body></html>");

    log::listing(uri, dir_count, file_count);
    log::request(method, uri, 200, ip);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(
            Full::new(Bytes::from(html))
                .map_err(|never| match never {})
                .boxed(),
        )
        .unwrap()
}

fn plain(code: StatusCode, text: &'static str) -> Resp {
    Response::builder()
        .status(code)
        .header(CONTENT_TYPE, "text/plain")
        .body(
            Full::new(Bytes::from_static(text.as_bytes()))
                .map_err(|never| match never {})
                .boxed(),
        )
        .unwrap()
}

fn sanitize(uri: &str, root: &Path) -> PathBuf {
    let decoded = percent_decode_str(uri)
        .decode_utf8_lossy();

    let trimmed = decoded.trim_start_matches('/');

    let joined = root.join(trimmed);

    if joined.exists() {
        return joined;
    }

    match joined.canonicalize() {
        Ok(p) if p.starts_with(root) => p,
        _ => root.to_path_buf(),
    }
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
