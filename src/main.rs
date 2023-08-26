use axum::{
    extract::{Extension, Query},
    http,
    http::status::StatusCode,
    response::{IntoResponse, Response},
};

use serde::Deserialize;

use std::{
    cell::RefCell,
    env::var,
    fmt::Display,
    fs::{File, OpenOptions},
    io::{Read, Write},
    net::SocketAddr,
    path::{Path, PathBuf},
    process::Command,
    sync::{mpsc::Receiver, Arc, Mutex},
};

const DEFUALT_HOST: &'static str = "127.0.0.1";

#[derive(Debug, Copy, Clone)]
struct Config {
    port: u16,
}

async fn ping(uri: http::Uri) -> impl IntoResponse {
    log::info!("ping got request ...");
    (http::status::StatusCode::OK, "").into_response()
}

async fn render(uri: http::Uri) -> impl IntoResponse {
    let content = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Demo</title>
                <meta charset="utf-8">
                <meta name="format-detection" content="telephone=no">
                <meta name="msapplication-tap-highlight" content="no">
                <meta name="viewport" content="user-scalable=no, initial-scale=1, maximum-scale=1, minimum-scale=1">
                <link rel="stylesheet" type="text/css" href="/file?tag=css">
            </head>
            <body class="test">
                {body}
            </body>
        </html>
    "#,
        body = "TEST BODY",
    );

    let url = css_inline::Url::parse(&format!("http://{DEFUALT_HOST}:{}", 3009)).ok();
    let inliner = css_inline::CSSInliner::options()
        .base_url(url)
        .load_remote_stylesheets(true)
        .build();
    let content = match inliner.inline(&content) {
        Ok(v) => v,
        Err(e) => {
            println!("failed to inline css style: {e:?}");
            content
        }
    };
    ([(axum::http::header::CONTENT_TYPE, "text/html")], content).into_response()
}

#[derive(Deserialize)]
enum FileTag {
    #[serde(rename = "css")]
    CSS,
    #[serde(rename = "js")]
    JS,
    #[serde(rename = "path")]
    Path,
}

#[derive(Deserialize)]
struct FileMeta {
    tag: FileTag,
    val: Option<String>,
}

async fn file(filemeta: Query<FileMeta>) -> impl IntoResponse {
    log::info!("got file request");
    let content = r#"
        .test {
            color: red;
           text-size: 32px;
        }
    "#;
    ([(axum::http::header::CONTENT_TYPE, "text/css")], content).into_response()
}

fn server(config: Config) {
    let config = Arc::new(config);
    let addr = format!("{DEFUALT_HOST}:{}", config.port)
        .parse::<SocketAddr>()
        .unwrap();
    log::info!("web server start to listen at {}", addr.to_string());
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(10)
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let app = axum::Router::new()
            .route("/", axum::routing::get(render))
            .route("/ping", axum::routing::get(ping))
            .route("/file", axum::routing::get(file));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
}

#[tokio::main]
async fn main() {
    let config = Config { port: 3009 };
    std::thread::spawn(move || server(config));
    log::info!("wait server up ...");
    let pingurl = format!("http://{DEFUALT_HOST}:{}/ping", config.port);
    while reqwest::get(&pingurl).await.is_err() {}
    log::info!("server started with configuration: {:?}", config);
    loop {}
}
