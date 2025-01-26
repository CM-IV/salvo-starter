use crate::middleware::{cors::cors_middleware, handle_404::handle_404};
use crate::routers::router;
use config::{init_db_conn, CFG};
use salvo::catcher::Catcher;
use salvo::prelude::*;
use salvo::server::ServerHandle;
use tokio::signal;
use tracing::info;
mod app_error;
mod app_writer;
mod config;
mod dtos;
mod entities;
mod middleware;
mod routers;
mod services;
mod utils;

#[tokio::main]
async fn main() {
    //At the same time, logs are only output to the terminal or file
    let _guard = clia_tracing_config::build()
        .filter_level(&CFG.log.filter_level)
        .with_ansi(CFG.log.with_ansi)
        .to_stdout(CFG.log.to_stdout)
        .directory(&CFG.log.directory)
        .file_name(&CFG.log.file_name)
        .rolling(&CFG.log.rolling)
        .init();

    init_db_conn().await;
    let router = router();
    let service: Service = router.into();
    let service = service.catcher(Catcher::default().hoop(handle_404)); //.hoop(_cors_handler).hoop(handle_404));
    let _cors_handler = cors_middleware();

    let acceptor = TcpListener::new(&CFG.server.address).bind().await;
    let server = Server::new(acceptor);
    let handle = server.handle();
    tokio::spawn(shutdown_signal(handle));
    server.serve(service).await;
}

async fn shutdown_signal(handle: ServerHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => info!("ctrl_c signal received"),
        _ = terminate => info!("terminate signal received"),
    }
    handle.stop_graceful(std::time::Duration::from_secs(60));
}
