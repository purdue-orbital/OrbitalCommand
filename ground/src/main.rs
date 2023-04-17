use actix_web::{post, App, HttpResponse, HttpServer};
use actix_web::web::Data;
use mimalloc::MiMalloc;
use radio::radio::Radio;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

struct State {
    radio: Radio
}

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(Data::new(State {
                radio: Radio::new().unwrap(),
            }))
            .service(launch)
            .service(abort)
            .service(cut)
            .service(actix_files::Files::new("/", "./dist").index_file("index.html"))
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
