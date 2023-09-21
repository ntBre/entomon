#![feature(let_chains)]

//! web ui for debugging torsion multiplicity issues

use actix_files::NamedFile;
use std::path::PathBuf;
use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    fs::{self, File},
    path::Path,
    sync::RwLock,
};

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use ligand::molecule::Molecule;
use openff_toolkit::qcsubmit::results::{Entry, ResultCollection};

/// DDE records loaded from a CSV file
struct Record {
    id: String,
    dde: f64,
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:12.8}", self.id, self.dde)
    }
}

/// load a data set from the JSON file specified by `filename` and return a map
/// of record_id -> cmiles pairs
fn load_dataset(filename: impl AsRef<Path>) -> HashMap<String, String> {
    let data = ResultCollection::parse_file(filename).unwrap();

    let map: HashMap<String, String> = data
        .into_entries()
        .map(
            |Entry {
                 record_id, cmiles, ..
             }| (record_id, cmiles),
        )
        .collect();
    map
}

fn load_records(filename: impl AsRef<Path>) -> Vec<Record> {
    let f = File::open(filename).unwrap();
    let mut rdr = csv::Reader::from_reader(f);
    rdr.records()
        .flatten()
        .map(|result| Record {
            id: result[0].to_owned(),
            dde: result[1].parse().unwrap(),
        })
        .collect()
}

fn load(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).unwrap()
}

type Datum = web::Data<State>;

async fn index(data: Datum, req: HttpRequest) -> impl Responder {
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
    let page = data.page;
    const PAGE_SIZE: usize = 200;
    for record in &records[page * PAGE_SIZE..(page + 1) * PAGE_SIZE] {
        writeln!(body, "<tr>").unwrap();
        write!(body, "<td><a href=/?id={0}>{}</a></td>", record.id).unwrap();
        for val in record.vals.iter() {
            write!(body, "<td>{val:.6}</td>").unwrap();
        }
        writeln!(body, "</tr>").unwrap();
    }
    writeln!(body, "</table>").unwrap();

    let index = load("static/index.html");
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

#[allow(unused)]
struct State {
    rows: RwLock<Vec<Row>>,
    names: RwLock<Vec<String>>,
    map: RwLock<HashMap<String, String>>,
    page: usize,
}

impl State {
    fn new(
        records: Vec<Row>,
        map: HashMap<String, String>,
        names: Vec<String>,
    ) -> Self {
        Self {
            rows: RwLock::new(records),
            names: RwLock::new(names),
            map: RwLock::new(map),
            page: 0,
        }
    }
}

// copy pasta from berry-patch
macro_rules! file_handlers {
    ($($name:ident => $path:expr$(,)*)*) => {
	$(
	    async fn $name(req: HttpRequest) -> actix_web::Result<NamedFile> {
		let dir = Path::new($path);
		let path: PathBuf = req.match_info()
		    .query("filename").parse().unwrap();
		Ok(NamedFile::open(dir.join(path))?)
	    }
	)*
    }
}

file_handlers! {
    css_file => "css/"
    js_file => "js/"
}

struct Row {
    id: String,
    vals: Vec<f64>,
}

fn build_rows(records: Vec<Vec<Record>>) -> Vec<Row> {
    let mut map = HashMap::new();
    let lr = records.len();
    for set in records {
        for record in set {
            map.entry(record.id).or_insert(Vec::new()).push(record.dde);
        }
    }

    let mut ret = Vec::new();
    for (id, vals) in map {
        if vals.len() == lr {
            ret.push(Row { id, vals });
        } else {
            eprintln!(
                "warning: omitting record {id} for {}/{lr} fields",
                vals.len()
            );
        }
    }
    ret
}

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
            .route("/css/{filename:.*}", web::get().to(css_file))
            .route("/js/{filename:.*}", web::get().to(js_file))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
