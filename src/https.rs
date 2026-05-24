use crate::cert;
use crate::fs::handle;
use crate::log;
use crate::Config;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use rustls::ServerConfig;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

pub async fn run(cfg: Config) {
    log::startup(&cfg);

    let (certs, key) = cert::load();

    let tls = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();

    let acceptor = TlsAcceptor::from(Arc::new(tls));

    let listener = TcpListener::bind(("0.0.0.0", cfg.port))
        .await
        .unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let acc = acceptor.clone();
        let c = cfg.clone();

        tokio::spawn(async move {
            match acc.accept(stream).await {
                Ok(tls) => {
                    let io = TokioIo::new(tls);

                    let service = service_fn(move |req| {
                        let cc = c.clone();
                        async move { handle(req, cc).await }
                    });

                    if let Err(e) = http1::Builder::new()
                        .keep_alive(true)
                        .serve_connection(io, service)
                        .await
                    {
                        log::error(&e.to_string());
                    }
                }
                Err(e) => log::error(&e.to_string()),
            }
        });
    }
}
