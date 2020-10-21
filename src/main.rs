mod reddit;

use crate::reddit::{hot_wholesome_meme, reddit_access_token};
use anyhow::{anyhow, Context, Error};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::*;
use logosaurus::{self, Logger, L_SHORT_FILE, L_STD};
use reqwest::Client as HttpClient;
use std::env;
use std::io;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logosaurus::init(
        Logger::builder(io::stderr())
            .set_level(LevelFilter::Info)
            .set_flags(L_STD | L_SHORT_FILE)
            .build(),
    )?;

    let reddit_client_id = env::var("REDDIT_CLIENT_ID").context("REDDIT_CLIENT_ID missing")?;
    let reddit_client_secret =
        env::var("REDDIT_CLIENT_SECRET").context("REDDIT_CLIENT_SECRET missing")?;

    let http = HttpClient::new();
    let token = reddit_access_token(&http, &reddit_client_id, &reddit_client_secret).await?;
    info!("{}", token);
    let meme = hot_wholesome_meme(&http, &token).await?;
    info!("{:?}", meme);

    let port = determine_port()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("listening on {}", addr);

    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(hello)) });
    let server = Server::bind(&addr).serve(service);

    server.await?;

    Ok(())
}

async fn hello(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(Body::from("hello, world!"))),
        (_, "/") => {
            let mut rsp = Response::default();
            *rsp.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            Ok(rsp)
        }
        _ => {
            let mut rsp = Response::default();
            *rsp.status_mut() = StatusCode::NOT_FOUND;
            Ok(rsp)
        }
    }
}

fn determine_port() -> Result<u16, Error> {
    match env::var("PORT") {
        Err(env::VarError::NotPresent) => Ok(3000),
        Err(env::VarError::NotUnicode(s)) => Err(anyhow!(env::VarError::NotUnicode(s))),
        Ok(p) => p.parse::<u16>().map_err(|e| anyhow!(e)),
    }
}
