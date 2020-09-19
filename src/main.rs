use actix_web::{middleware, web, App, HttpResponse, HttpRequest, HttpServer, Error};
use diesel::{SqliteConnection};
use diesel::r2d2::{self,ConnectionManager};

// the next old macro import style is needed for schema.rs
#[macro_use]
extern crate diesel;

mod models;
mod db;

async fn map_handler(req: HttpRequest, pool: web::Data<db::Pool>) -> Result<HttpResponse, Error> {
    let keystr =  req.match_info().get("map").unwrap().to_owned();

    let conn = pool.get().expect("couldn't get db connection from pool");
    
    let map = web::block(move || db::actions::find_map_by_keystr(&keystr, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    if let Some(map) = map {
        Ok(HttpResponse::Ok().body(format!("You found {}!", map.keystr)))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // set environment variables
    std::env::set_var("RUST_LOG", "actix_web=info");
    dotenv::dotenv().ok(); // ignore errors, i.e. file is not found

    // set global logging (uses RUST_LOG env variable)
    env_logger::init();

    // create r2d2 pool to DB
    let connspec = std::env::var("DATABASE_URL").expect("no env variable DATABASE_URL");
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
    let pool: db::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let bind = "127.0.0.1:8080";
    println!("Starting server at: {}", &bind);

    HttpServer::new(move || {
        App::new()
        .data(pool.clone())
        .wrap(middleware::Logger::default())
        .service(
            web::scope("/maps")
                .route("/{map}", web::get().to(map_handler))
        )
    })
    .bind(&bind)?
    .run()
    .await
}