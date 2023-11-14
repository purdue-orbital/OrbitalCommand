// #![deny(clippy::unwrap_used)]
// #![deny(clippy::expect_used)]

use actix_web::{post, get, App, HttpResponse, HttpServer};
use actix_web::web::{Data, Json};
use async_mutex::Mutex;
use common::{MessageToLaunch, MessageToGround, Vec3};
use radio::RadioStream;
use serde::Serialize;

struct State {
    radio: Mutex<RadioStream>
}

#[post("/launch")]
async fn launch(state: Data<State>) -> actix_web::Result<HttpResponse> {
    state.radio.lock().await.transmit(&std::convert::TryInto::<Vec<_>>::try_into(MessageToLaunch::Launch).unwrap()).unwrap();
    Ok(HttpResponse::Ok().finish())
}

#[post("/abort")]
async fn abort(state: Data<State>) -> actix_web::Result<HttpResponse> {
    state.radio.lock().await.transmit(&std::convert::TryInto::<Vec<_>>::try_into(MessageToLaunch::Abort).unwrap()).unwrap();
    Ok(HttpResponse::Ok().finish())
}

#[post("/cut")]
async fn cut(state: Data<State>) -> actix_web::Result<HttpResponse> {
    state.radio.lock().await.transmit(&std::convert::TryInto::<Vec<_>>::try_into(MessageToLaunch::Cut).unwrap()).unwrap();
    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize)]
struct Telemetry {
    imu: Option<ImuTelemetry>,
    gps: Option<GpsTelemetry>,
}

#[derive(Debug, Serialize)]
struct ImuTelemetry {
    temperature: f64,
    acceleration: Vec3,
    gyro: Vec3,
}

#[derive(Debug, Serialize)]
struct GpsTelemetry {
    altitude: f64,
    latitude: f64,
    longitude: f64,
    velocity: f64,
    heading: f64,
}

#[get("/telemetry")]
async fn telemetry(state: Data<State>) -> actix_web::Result<Json<Telemetry>> {
    let messages = state.radio.lock().await.receive_frames().unwrap();
    let mut imu = None;
    let mut gps = None;
    for msg in messages.iter().rev() {
        if let Ok(msg) = MessageToGround::try_from(msg.data.as_slice()) {
            match msg {
                MessageToGround::ImuTelemetry { temperature, acceleration, gyro } => {let _ = imu.insert(ImuTelemetry {
                    temperature,
                    acceleration,
                    gyro,
                });},
                MessageToGround::GpsTelemetry { altitude, latitude, longitude, velocity, heading } => {
                    let _ = gps.insert(GpsTelemetry { altitude, latitude, longitude, velocity, heading });
                }
            }
        }
    }

    Ok(Json(Telemetry { imu, gps }))
}

// #[post("/update")]
// async fn update(state: Data<State>) -> actix_web::Result<HttpResponse> {
//     state.radio.lock().await.transmit(&std::convert::TryInto::<Vec<_>>::try_into(MessageToLaunch::Update).unwrap()).unwrap();
//     Ok(HttpResponse::Ok().finish())
// }

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

    HttpServer::new(move || {
        let state = state.clone();
        App::new()
            .app_data(state)
            .service(launch)
            .service(abort)
            .service(cut)
            .service(telemetry)
            // .service(update)
            .service(map_token)
            .service(actix_files::Files::new("/", "./dist").index_file("index.html"))
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
