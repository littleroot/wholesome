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
use std::sync::Arc;
use std::time::Duration;

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

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

    let wholesome_svc = Arc::new(WholesomeService {
        reddit_client_id,
        reddit_client_secret,
        http: HttpClient::builder().timeout(HTTP_TIMEOUT).build().unwrap(),
    });

    let port = determine_port()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let make_service = make_service_fn(|_| {
        let w = Arc::clone(&wholesome_svc);
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let w = Arc::clone(&w);
                handle_request(req, w)
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_service);
    info!("listening on {}", addr);
    server.await?;

    Ok(())
}

struct WholesomeService {
    reddit_client_id: String,
    reddit_client_secret: String,
    http: HttpClient,
}

async fn handle_request(
    req: Request<Body>,
    w: Arc<WholesomeService>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (_, "/") => root(req, w).await,
        _ => {
            let rsp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
            Ok(rsp)
        }
    }
}

async fn root(
    req: Request<Body>,
    w: Arc<WholesomeService>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let token =
                reddit_access_token(&w.http, &w.reddit_client_id, &w.reddit_client_secret).await;
            if let Err(e) = token {
                error!("fetch reddit access token: {}", e);
                let rsp = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap();
                return Ok(rsp);
            }

            let meme = hot_wholesome_meme(&w.http, &token.unwrap()).await;
            if let Err(e) = meme {
                error!("fetch wholesome meme: {}", e);
                let rsp = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap();
                return Ok(rsp);
            }

            info!("{:?}", meme.unwrap());
            Ok(Response::new(Body::from("hello, world!")))
        }
        (_, "/") => {
            let rsp = Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::empty())
                .unwrap();
            Ok(rsp)
        }
        _ => unreachable!(),
    }
}

fn determine_port() -> Result<u16, Error> {
    match env::var("PORT") {
        Err(env::VarError::NotPresent) => Ok(3000),
        Err(env::VarError::NotUnicode(s)) => Err(anyhow!(env::VarError::NotUnicode(s))),
        Ok(p) => p.parse::<u16>().map_err(|e| anyhow!(e)),
    }
}
