mod reddit;

use crate::reddit::{hottest_wholesome_meme, reddit_access_token, Post};
use anyhow::{anyhow, Context, Error};
use horrorshow::helper::doctype;
use horrorshow::html;
use horrorshow::prelude::*;
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

    let wholesome_svr = Arc::new(WholesomeServer {
        reddit_client_id,
        reddit_client_secret,
        http: HttpClient::builder().timeout(HTTP_TIMEOUT).build().unwrap(),
    });

    let port = determine_port()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let make_service = make_service_fn(|_| {
        let w = Arc::clone(&wholesome_svr);
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

struct WholesomeServer {
    reddit_client_id: String,
    reddit_client_secret: String,
    http: HttpClient,
}

async fn handle_request(
    req: Request<Body>,
    s: Arc<WholesomeServer>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (_, "/") => root(req, s).await,
        _ => {
            let rsp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("not found"))
                .unwrap();
            Ok(rsp)
        }
    }
}

fn render_root_template(post: Post) -> String {
    let style = r#"
html, body {
    margin: 0;
    padding: 0;
}
html {
    font-family: sans-serif;
}
body {
    margin: 0 15px;
}
p.title {
    font-size: larger;
    margin-bottom: 3em;
    display: flex;
    justify-content: center;
}
a.meme {
    display: flex;
    justify-content: center;
}
img.meme {
    max-width: 100%;
}
"#;

    (html! {
        : doctype::HTML;
        html {
            head {
                meta(charset="UTF-8");
                meta(name="viewport", content="width=device-width");
                title {
                    : "A wholesome meme";
                }
                style {
                    : style
                }
            }
            body {
                main {
                    p(class="title") {
                        a(href=format!("https://reddit.com/{}", &post.permalink)) {
                            : &post.title;
                        }
                    }
                    // TODO: handle Option on post.url
                    a(class="meme", href=&post.url) {
                        img(class="meme", src=&post.url, alt="The hottest wholesome meme on Reddit right now");
                    }
                }
            }
        }
    })
    .into_string()
    .unwrap()
}

async fn root(req: Request<Body>, s: Arc<WholesomeServer>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let token =
                reddit_access_token(&s.http, &s.reddit_client_id, &s.reddit_client_secret).await;
            if let Err(e) = token {
                error!("fetch reddit access token: {}", e);
                let rsp = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(
                        "internal server error: failed to fetch reddit access token",
                    ))
                    .unwrap();
                return Ok(rsp);
            }

            let meme = hottest_wholesome_meme(&s.http, &token.unwrap()).await;
            if let Err(e) = meme {
                error!("fetch wholesome meme: {}", e);
                let rsp = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(
                        "internal server error: failed to fetch wholesome meme",
                    ))
                    .unwrap();
                return Ok(rsp);
            }

            Ok(Response::new(Body::from(render_root_template(
                meme.unwrap(),
            ))))
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
