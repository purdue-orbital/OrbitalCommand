use actix_web::{post, get, App, HttpResponse, HttpServer, Either};
use actix_web::Either::{Left, Right};
use actix_web::web::{Data, Json};
use async_mutex::Mutex;
use mimalloc::MiMalloc;
use common::Message;
use radio::RadioStream;
use serde::Serialize;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

struct State {
    radio: Mutex<RadioStream>
}

#[post("/launch")]
async fn launch(state: Data<State>) -> actix_web::Result<HttpResponse> {
    state.radio.lock().await.transmit(Message::Launch.into()).unwrap();
    Ok(HttpResponse::Ok().finish())
}

#[post("/abort")]
async fn abort(state: Data<State>) -> actix_web::Result<HttpResponse> {
    state.radio.lock().await.transmit(Message::Abort.into()).unwrap();
    Ok(HttpResponse::Ok().finish())
}

#[post("/cut")]
async fn cut(state: Data<State>) -> actix_web::Result<HttpResponse> {
    state.radio.lock().await.transmit(Message::Cut.into()).unwrap();
    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize)]
struct Telemetry {
    pos: Vec<f64>,
    acc: Vec<f64>,
    temp: f64,
}

#[get("/telemetry")]
async fn telemetry(state: Data<State>) -> actix_web::Result<Either<Json<Telemetry>, HttpResponse>> {
    let messages = state.radio.lock().await.read().unwrap();
    for msg in messages.iter().rev() {
        if let Ok(msg) = Message::try_from(msg.as_slice()) {
            return match msg {
                Message::Telemetry { temperature, gps, acceleration } => Ok(Left(Json(Telemetry {
                    pos: vec![gps.x, gps.y, gps.z],
                    acc: vec![acceleration.x, acceleration.y, acceleration.z],
                    temp: temperature,
                }))),
                _ => {}
            };
        }
    }

    Ok(Right(HttpResponse::NotFound().finish()))
}

#[post("/update")]
async fn update(state: Data<State>) -> actix_web::Result<HttpResponse> {
    state.radio.transmit(Message::Update.into()).unwrap();
    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize)]
struct MapToken {
    token: String,
}

#[get("/map_token")]
async fn map_token() -> actix_web::Result<Json<MapToken>> {
    Ok(Json(MapToken {
        token: option_env!("MAPBOX_TOKEN").unwrap_or("NO_TOKEN").to_string(),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = Data::new(State {
        radio: Mutex::new(RadioStream::new().unwrap()),
    });

    HttpServer::new(|| {
        App::new()
            .app_data(state)
            .service(launch)
            .service(abort)
            .service(cut)
            .service(telemetry)
            .service(update)
            .service(map_token)
            .service(actix_files::Files::new("/", "./dist").index_file("index.html"))
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
