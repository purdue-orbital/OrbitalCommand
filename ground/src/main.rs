use actix_web::{post, get, App, HttpResponse, HttpServer};
use actix_web::web::Data;
use mimalloc::MiMalloc;
//use radio::radio::Radio;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// struct State {
//     radio: Pipeline
// }

#[post("/launch")]
async fn launch() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

#[post("/abort")]
async fn abort() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

#[post("/cut")]
async fn cut() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

#[get("/telemetry")]
async fn telemetry() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().finish())
}

#[post("/update")]
async fn update() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().finish())
}

#[get("/map_token")]
async fn map_token() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::NotImplemented().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let state = Data::new(State {
    //     radio: Pipeline::new(915e6, 100e3).unwrap(),
    // });

    HttpServer::new(|| {
        App::new()
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
