use actix_web::{middleware, web, error, http::header, guard, get, App, HttpResponse, HttpRequest, HttpServer, Error};
use actix_files as fs;
use diesel::{SqliteConnection};
use diesel::r2d2::{self,ConnectionManager};
use std::path::{Path,PathBuf};
use std::fs::File;
use serde::{Serialize, Deserialize};
use regex::Regex;
use std::io::prelude::*;
use std::sync::atomic::{AtomicI32, Ordering};

// old macro import style is needed for schema.rs
#[macro_use]
extern crate diesel;

mod models;
mod db;

async fn get_map(req: &HttpRequest, pool: &db::Pool) -> Result<Option<models::Map>, Error> {
    let keystr =  req.match_info().get("mapid").unwrap().to_owned();
    let conn = pool.get().expect("couldn't get db connection from pool");
    let map = web::block(move || db::actions::find_map_by_keystr(&keystr, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;
    Ok(map)
}

fn internal_error() -> Error {
    error::ErrorInternalServerError(std::io::Error::new(std::io::ErrorKind::Other, "Internal Error"))
}

struct PointIdCounter {
    count: AtomicI32,
}

#[derive(Serialize)]
struct MapData {
    img_url: String,
    img_width: u32,
    img_height: u32,
}

async fn get_map_info(map: &models::Map, path: &Path) -> Result<MapData, Error> {
    let contents = {
        let path: PathBuf = [path, Path::new("maps"),  Path::new(&map.fpath), Path::new("ImageProperties.xml")].iter().collect();
        println!("Looking for file {:?}", path);
        let mut file = 
            web::block(move || File::open(path))
            .await
            .map_err(|e| match e {
                error::BlockingError::Error(internal) => error::ErrorNotFound(internal),
                _ => e.into()
            })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        contents
    };

    let img_width: u32 = {
        let width_regex: Regex = Regex::new(r#"WIDTH="(\d+)""#).unwrap();
        width_regex.captures(&contents[..]).ok_or_else(internal_error)?
            .get(1).ok_or_else(internal_error)?
            .as_str().parse().unwrap()
    };

    let img_height: u32 = {
        let height_regex: Regex = Regex::new(r#"HEIGHT="(\d+)""#).unwrap();
        height_regex.captures(&contents[..]).ok_or_else(internal_error)?
            .get(1).ok_or_else(internal_error)?
            .as_str().parse().unwrap()
    };

    let img_url: PathBuf = ["maps", &map.fpath[..], ""].iter().collect();
    let img_url = img_url.to_str().ok_or_else(internal_error)?.to_owned();

    Ok(MapData {
        img_url,
        img_height,
        img_width,
    })
}

async fn data_json(req: HttpRequest, pool: web::Data<db::Pool>, paths: web::Data<DataPaths>) -> Result<HttpResponse, Error> {
    println!("hey?");
    let map = get_map(&req, pool.as_ref()).await?;
    if let Some(map) = map {
        println!("map is {:?}", map);
        let body = serde_json::to_string(&get_map_info(&map, &paths.data_path).await?)?;
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
        )
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

#[derive(Serialize)]
struct CreatePointResponse {
    id: i64,
}

async fn create_point(req: HttpRequest, payload: web::Json<Vec<f32>>, pool: web::Data<db::Pool>, point_id_factory: web::Data<PointIdCounter>) -> Result<HttpResponse, Error> {
    let map = get_map(&req, pool.as_ref()).await?;
    if let Some(map) = map {

        let point_id = point_id_factory.count.fetch_add(1, Ordering::SeqCst);

        //let coordinates = web::Json<Vec<f64>>::from(req);
        let conn = pool.get().expect("couldn't get db connection from pool");

        let coordx = *payload.get(0).ok_or_else(internal_error)?;
        let coordy = *payload.get(1).ok_or_else(internal_error)?;

        web::block(move || db::actions::insert_point(&models::Point{ 
            id: point_id,
            mapid: map.id,
            coordx,
            coordy,
            title: None,
            body: None,
        }, &conn)).await?;

        let body = serde_json::to_string(&CreatePointResponse{id: point_id as i64})?;
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
        )
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}


#[derive(Serialize)]
struct GetPointsResponse {
    id: i32,
    coordinate: (f32, f32),
    title: String,
    descr: String,
}

impl From<models::Point> for GetPointsResponse {
    fn from(point: models::Point) -> Self {
        GetPointsResponse {
            id: point.id,
            coordinate: (point.coordx, point.coordy),
            title: point.title.unwrap_or_else(String::new),
            descr: point.body.unwrap_or_else(String::new),
        }
    }
}

async fn get_points(req: HttpRequest, pool: web::Data<db::Pool>) -> Result<HttpResponse, Error> {
    let keystr =  req.match_info().get("mapid").unwrap().to_owned();
    let conn = pool.get().expect("couldn't get db connection from pool");
    let points = web::block(move || db::actions::get_points_in_map(&keystr, &conn)).await?;
    let points: Vec<GetPointsResponse> = points.into_iter().map(|e| e.into()).collect();
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&points)?)
    )
}

#[derive(Deserialize)]
struct DeletePointRequest {
    id: i32
}

async fn delete_point(req: HttpRequest, payload: web::Json<DeletePointRequest>, pool: web::Data<db::Pool>) -> Result<HttpResponse, Error> {
    let keystr =  req.match_info().get("mapid").unwrap().to_owned();
    let conn = pool.get().expect("couldn't get db connection from pool");
    web::block(move || db::actions::delete_points_in_map(&keystr, payload.id, &conn)).await?;
    Ok(HttpResponse::Ok()
        .finish()
    )
}

#[derive(Deserialize)]
struct ModifyPointRequest {
    id: i32,
    title: String,
    descr: String,
}

impl Into<models::PointUpdate> for ModifyPointRequest {
    fn into(self) -> models::PointUpdate {
        models::PointUpdate{
            id: self.id,
            title: if self.title == "" { Some(None) } else { Some(Some(self.title)) },
            body: if self.descr == "" { Some(None) } else { Some(Some(self.descr)) },
        }
    }
}

async fn modify_point(req: HttpRequest, payload: web::Json<ModifyPointRequest>, pool: web::Data<db::Pool>) -> Result<HttpResponse, Error> {
    let keystr =  req.match_info().get("mapid").unwrap().to_owned();
    let conn = pool.get().expect("couldn't get db connection from pool");
    web::block(move || db::actions::modify_point_in_map(&keystr, &payload.0.into(), &conn)).await?;
    Ok(HttpResponse::Ok()
        .finish()
    )
}

#[get("/")]
async fn redirect_index(req: HttpRequest) -> Result<HttpResponse, Error> {
    let path = req.uri().path();
    let path = format!("{}/index.html", path);
    println!("Redirect to {}", path);
    Ok(HttpResponse::Found()
        .header(header::LOCATION,  path)
        .finish()
    )
}

#[derive(Clone)]
struct DataPaths
{
    www_path: PathBuf,
    data_path: PathBuf,
}

#[get("/{filename:.*}/")]
async fn index(req: HttpRequest, paths: web::Data<DataPaths>) -> Result<fs::NamedFile, Error> {

    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let file = fs::NamedFile::open(paths.www_path.join(path))?;
    Ok(file)
}

#[get("/maps/{filename:.*}/")]
async fn map_imgs(req: HttpRequest, paths: web::Data<DataPaths>) -> Result<fs::NamedFile, Error> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let path: PathBuf = [paths.data_path.as_path(), Path::new("maps"), path.as_path()].iter().collect();
    println!("trying to open {:?}", path);
    let file = fs::NamedFile::open(path)?;
    Ok(file)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // set environment variables
    std::env::set_var("RUST_LOG", "actix_web=info");
    dotenv::dotenv().ok(); // ignore errors, i.e. file is not found

    // set global logging (uses RUST_LOG env variable)
    env_logger::init();

    // setup server paths
    let paths = DataPaths {
        www_path: PathBuf::from(std::env::var("MAPSERVER_WWW_PATH").expect("no env variable MAPSERVER_WWW_PATH")),
        data_path: PathBuf::from(std::env::var("MAPSERVER_DATA_PATH").expect("no env variable MAPSERVER_DATA_PATH")),
    };

    // create r2d2 pool to DB
    let connspec = std::env::var("MAPSERVER_DATABASE_URL").expect("no env variable MAPSERVER_DATABASE_URL");
    println!("Connecting to DB at: {}", connspec);
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
    let pool: db::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let id_counter = web::Data::new(PointIdCounter {
        count: AtomicI32::new( db::actions::count_points(&pool.get().unwrap()).unwrap() ),
    });

    let bind = std::env::var("MAPSERVER_SERVER_ADDR").expect("no env variable MAPSERVER_SERVER_ADDR");
    println!("Starting server at: {}", &bind);

    HttpServer::new(move || {
        App::new()
        .data(pool.clone())
        .data(paths.clone())
        .app_data(id_counter.clone())
        .app_data(web::JsonConfig::default()
            .limit(4096)
            .error_handler(|err, _req| {
                error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            })
        )
        .wrap(middleware::Logger::default())
        .wrap(middleware::NormalizePath::default())
        .service(
            web::scope(r"/map/{mapid:\w+}")
                // API
                .service(web::resource("/create_point/")
                    .guard(guard::Header("accept", "application/json"))
                    .guard(guard::Header("content-type", "application/json"))
                    .route(web::post().to(create_point))
                )
                .service(web::resource("/data.json/")
                    .guard(guard::Header("accept", "application/json"))
                    .route(web::get().to(data_json))
                )
                .service(web::resource("/get_points/")
                    .guard(guard::Header("accept", "application/json"))
                    .route(web::get().to(get_points))
                )
                .service(web::resource("/delete_point/")
                    .guard(guard::Header("content-type", "application/json"))
                    .route(web::delete().to(delete_point))
                )
                .service(web::resource("/modify_point/")
                    .guard(guard::Header("content-type", "application/json"))
                    .route(web::put().to(modify_point))
                )

                // static resources
                .service(redirect_index)
                .service(map_imgs)
                .service(index)
        )
    })
    .bind(&bind)?
    .run()
    .await
}