use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{collections::HashMap, sync::RwLock};

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use ligand::molecule::Molecule;

use crate::data::Row;

type Datum = web::Data<State>;

#[derive(Debug, Deserialize)]
pub enum Info {
    Show(Vec<usize>),
    All,
}

#[derive(Serialize)]
pub struct State {
    rows: RwLock<Vec<Row>>,
    names: RwLock<Vec<String>>,
    map: RwLock<HashMap<String, String>>,
    query: RwLock<String>,
}

impl State {
    pub fn new(
        records: Vec<Row>,
        map: HashMap<String, String>,
        names: Vec<String>,
    ) -> Self {
        Self {
            rows: RwLock::new(records),
            names: RwLock::new(names),
            map: RwLock::new(map),
            query: RwLock::new(String::new()),
        }
    }
}

pub async fn file_handler(req: HttpRequest) -> impl Responder {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let mut body = "";
    match path.to_str().unwrap() {
        "style.css" => body = include_str!("../css/style.css"),
        "sort.js" => body = include_str!("../js/sort.js"),
        "query.js" => body = include_str!("../js/query.js"),
        s => eprintln!("unrecognized file {s}"),
    };
    HttpResponse::Ok().body(body)
}

pub async fn get_data(data: Datum) -> impl Responder {
    HttpResponse::Ok().body(serde_json::to_string(&data).unwrap())
}

pub async fn set_query(data: Datum, info: web::Json<String>) -> impl Responder {
    let mut data = data.query.write().unwrap();
    *data = info.0;
    HttpResponse::Ok()
}

pub async fn api(data: Datum, info: web::Json<Info>) -> impl Responder {
    let mut rows = data.rows.write().unwrap();
    match info.0 {
        Info::Show(to_show) => {
            for (i, row) in rows.iter_mut().enumerate() {
                row.show = to_show.contains(&i);
            }
        }
        Info::All => {
            for row in rows.iter_mut() {
                row.show = true;
            }
        }
    };
    HttpResponse::Ok()
}

pub async fn index(data: Datum, req: HttpRequest) -> impl Responder {
    use std::fmt::Write;
    let mut body = String::new();
    let records = data.rows.read().unwrap();
    let names = data.names.read().unwrap();
    writeln!(
        body,
        r#"<table id="myTable2">
            <tr>
            <th onclick="sortTable(0)">Record ID</th>"#
    )
    .unwrap();
    for (i, name) in names.iter().enumerate() {
        writeln!(body, "<th onclick=\"sortTable({})\">{name}</th>", i + 1)
            .unwrap();
    }
    writeln!(body, "</tr>").unwrap();
    const PAGE_LIMIT: usize = 200;
    let mut shown = 0;
    for row in records.iter() {
        if !row.show {
            continue;
        }
        if shown >= PAGE_LIMIT {
            break;
        }
        write!(body, "<tr>\n<td><a href=/?id={0}>{}</a></td>", row.id).unwrap();
        for val in row.vals.iter() {
            write!(body, "<td>{val:.6}</td>").unwrap();
        }
        writeln!(body, "</tr>").unwrap();
        shown += 1;
    }
    writeln!(body, "</table>").unwrap();

    let index = include_str!("../static/index.html")
        .replace("{{query}}", &data.query.read().unwrap());
    let query = req.query_string();
    let index = if !query.is_empty() {
        let map = data.map.read().unwrap();
        let sp: Vec<_> = query.split('=').collect();
        // TODO cache the generated SVG files
        let mol = Molecule::from_mapped_smiles(&map[sp[1]]).unwrap();
        index.replace("{{pic}}", &mol.to_svg())
    } else {
        index.replace("{{pic}}", "")
    };

    HttpResponse::Ok().body(index.replace("{{body}}", &body))
}
