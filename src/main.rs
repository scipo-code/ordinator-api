mod models;
mod data_processing;

use calamine::{Xlsx};
use std::io::BufReader;
use std::fs::File;
use crate::models::scheduling_environment::{self, SchedulingEnvironment};

use std::path::Path;
use std::env;
use crate::data_processing::sources::excel::load_data_file;


use actix_web::{get, web, App, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let args: Vec<String> = env::args().collect();
    let xlsx: Xlsx<BufReader<File>>;
    if args.len() > 1 {
        let file_path = Path::new(&args[1]);
        let scheduling_environment = load_data_file(file_path).expect("Could not load data file.");
        println!("{}", scheduling_environment)
    } else {
        println!("Please provide the data file as an argument.");
    }
    // let scheduling_environment = SchedulingEnvironment::initialize_from_sources(work_orders, worker_environment);

    HttpServer::new(|| {
        App::new().service(greet)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", name)
}