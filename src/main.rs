use config::CFG;
use log::info;
use router::create_router;
use salvo::prelude::*;

mod config;
mod result;
mod router;
mod utils;
mod db;

#[tokio::main]
async fn main() {
    let _guard = clia_tracing_config::build()
        .filter_level(&CFG.log.filter_level)
        .with_ansi(CFG.log.with_ansi)
        .to_stdout(CFG.log.to_stdout)
        .with_source_location(false)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .directory(&CFG.log.directory)
        .file_name(&CFG.log.file_name)
        .rolling(&CFG.log.rolling)
        .init();
    info!("Starting server");
    let acceptor = TcpListener::new(&CFG.server.address).bind().await;
    let server = Server::new(acceptor);
    #[allow(unused_variables)] // 防止开发时候报WARN
    let handle = server.handle();
    // 初始化路由
    let router = create_router();
    // 优雅关机
    // 生产环境再启用
    // #[cfg(not(debug_assertions))] // 这个代表非debug模式
    // tokio::spawn(async move {
    //     shutdown_signal().await;
    //     handle.stop_graceful(None);
    // });
    server.serve(router).await;
}
