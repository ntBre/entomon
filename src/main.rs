#![feature(let_chains)]

//! web ui for debugging torsion multiplicity issues

use data::{build_rows, load_dataset, load_records};
use serde::Deserialize;
use server::{api, get_data, index, set_query, State};
use std::{error::Error, path::Path};

use actix_web::{web, App, HttpServer};

use crate::server::file_handler;

mod data;
mod server;

#[derive(Deserialize)]
struct Config {
    records: Vec<String>,
    names: Vec<String>,
    dataset: String,
}

impl Config {
    fn load(path: impl AsRef<Path>) -> Self {
        let s = std::fs::read_to_string(path).unwrap();
        toml::from_str(&s).unwrap()
    }
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Some(inp) = std::env::args().nth(1) else {
        eprintln!("Usage: entomon INPUT_FILE");
        std::process::exit(1);
    };

    let Config {
        records,
        names,
        dataset,
    } = Config::load(inp);

    let records = records.into_iter().map(load_records).collect();
    let rows = build_rows(records);
    let map = load_dataset(dataset);

    let state = web::Data::new(State::new(rows, map, names));

    const ADDR: &str = "127.0.0.1";
    const PORT: u16 = 8080;
    println!("serving on {ADDR}:{PORT}");
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(index))
            .route("/api", web::post().to(api))
            .route("/api", web::get().to(get_data))
            .route("/set-query", web::post().to(set_query))
            .route("/css/{filename:.*}", web::get().to(file_handler))
            .route("/js/{filename:.*}", web::get().to(file_handler))
    })
    .bind((ADDR, PORT))?
    .run()
    .await?;

    Ok(())
}
