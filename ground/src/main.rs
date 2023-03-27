use actix_web::{post, App, HttpResponse, HttpServer};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[post("/launch")]
async fn launch() -> actix_web::Result<HttpResponse> {
    todo!()
}

#[post("/abort")]
async fn abort() -> actix_web::Result<HttpResponse> {
    todo!()
}

#[post("/cut")]
async fn cut() -> actix_web::Result<HttpResponse> {
    todo!()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
