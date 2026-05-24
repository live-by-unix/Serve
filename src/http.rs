use crate::fs::handle;
use crate::log;
use crate::Config;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

pub async fn run(cfg: Config) {
    log::startup(&cfg);

    let listener = TcpListener::bind(("0.0.0.0", cfg.port))
        .await
        .unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        let c = cfg.clone();

        tokio::spawn(async move {
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
        });
    }
}
