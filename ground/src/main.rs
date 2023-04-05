use actix_web::{post, App, HttpResponse, HttpServer};
use actix_web::web::Data;
use mimalloc::MiMalloc;
use radio::pipeline::Pipeline;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

struct State {
    radio: Pipeline
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
    let state = Data::new(State {
        radio: Pipeline::new(915e6, 100e3).unwrap(),
    });

    HttpServer::new(|| {
        App::new()
            .service(launch)
            .service(abort)
            .service(cut)
            .service(actix_files::Files::new("/", "./dist").index_file("index.html"))
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
