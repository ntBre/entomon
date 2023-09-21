#![feature(let_chains)]

//! web ui for debugging torsion multiplicity issues

use data::{build_rows, load_dataset, load_records};
use server::{api, css_file, get_data, index, js_file, set_query, State};
use std::error::Error;

use actix_web::{web, App, HttpServer};

mod data;
mod server;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let records = [
        "/home/brent/omsf/projects/benchmarking/output/industry/tm/dde.csv",
        "/home/brent/omsf/projects/benchmarking/output/industry/sage-tm/dde.csv",
        "/home/brent/omsf/projects/benchmarking/output/industry/sage-2.1.0/dde.csv",
    ].into_iter().map(load_records).collect();

    let names = vec!["TM".to_owned(), "Sage-TM".to_owned(), "Sage".to_owned()];
    let rows = build_rows(records);

    let map = load_dataset(
        "/home/brent/omsf/projects/benchmarking/datasets/industry.json",
    );

    let state = web::Data::new(State::new(rows, map, names));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(index))
            .route("/api", web::post().to(api))
            .route("/api", web::get().to(get_data))
            .route("/set-query", web::post().to(set_query))
            .route("/css/{filename:.*}", web::get().to(css_file))
            .route("/js/{filename:.*}", web::get().to(js_file))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
