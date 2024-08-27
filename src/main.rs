use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use async_stream::stream;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tokio::time::{self, Duration, Instant};

#[derive(Serialize, Deserialize, Clone)]
struct RequestInfo {
    method: String,
    headers: String,
    body: String,
}

type SharedState = Arc<RwLock<HashMap<String, RequestInfo>>>;
type Channels = Arc<RwLock<HashMap<String, (broadcast::Sender<RequestInfo>, Instant)>>>;

#[get("/events/{identification}")]
async fn get_events(
    req: HttpRequest,
    state: web::Data<SharedState>,
    channels: web::Data<Channels>,
    path: web::Path<String>,
) -> impl Responder {
    let identification = path.into_inner();
    let info = RequestInfo {
        method: req.method().to_string(),
        headers: format!("{:?}", req.headers()),
        body: format!("Identification: {}", identification),
    };

    {
        let mut state = state.write().unwrap();
        state.insert(identification.clone(), info.clone());
    }

    let tx = {
        let mut channels = channels.write().unwrap();
        channels
            .entry(identification.clone())
            .or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(100);
                (tx, Instant::now())
            })
            .0
            .clone()
    };

    let mut rx = tx.subscribe();
    let stream = stream! {
        while let Ok(info) = rx.recv().await {
            yield Ok::<_, actix_web::Error>(web::Bytes::from(format!("data: {}\n\n", serde_json::to_string(&info).unwrap())));
        }
    };

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(stream)
}

async fn callback_handler(
    req: HttpRequest,
    body: String,
    state: web::Data<SharedState>,
    channels: web::Data<Channels>,
    path: web::Path<String>,
) -> impl Responder {
    let identification = path.into_inner();
    let info = RequestInfo {
        method: req.method().to_string(),
        headers: format!("{:?}", req.headers()),
        body: format!("Body: {}", body),
    };

    {
        let mut state = state.write().unwrap();
        state.insert(identification.clone(), info.clone());
    }

    if let Some((tx, _)) = channels.read().unwrap().get(&identification) {
        let _ = tx.send(info.clone());
    }

    HttpResponse::Ok().json(info)
}

#[get("/latest/{identification}")]
async fn get_latest(state: web::Data<SharedState>, path: web::Path<String>) -> impl Responder {
    let identification = path.into_inner();
    let state = state.read().unwrap();
    if let Some(info) = state.get(&identification) {
        HttpResponse::Ok().json(info)
    } else {
        HttpResponse::NotFound().body(format!(
            "No data available for identification: {}",
            identification
        ))
    }
}

async fn cleanup_channels(channels: web::Data<Channels>) {
    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let mut channels = channels.write().unwrap();
        let now = Instant::now();
        channels
            .retain(|_, (_, last_used)| now.duration_since(*last_used) < Duration::from_secs(300));
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Sets the host address
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Sets the port number
    #[arg(long, default_value = "18686")]
    port: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let state = web::Data::new(Arc::new(RwLock::new(HashMap::<String, RequestInfo>::new())));
    let channels = web::Data::new(Arc::new(RwLock::new(HashMap::<
        String,
        (broadcast::Sender<RequestInfo>, Instant),
    >::new())));

    let channels_clone = channels.clone();
    tokio::spawn(async move {
        cleanup_channels(channels_clone).await;
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(channels.clone())
            .service(get_events)
            .service(
                web::resource("/callback/{identification}")
                    .route(web::post().to(callback_handler))
                    .route(web::put().to(callback_handler))
                    .route(web::patch().to(callback_handler))
                    .route(web::get().to(callback_handler))
                    .route(web::delete().to(callback_handler)),
            )
            .service(get_latest)
    })
    .bind(format!("{}:{}", args.host, args.port))?
    .run()
    .await
}
