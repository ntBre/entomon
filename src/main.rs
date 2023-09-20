#![feature(let_chains)]

//! web ui for debugging torsion multiplicity issues

use actix_files::NamedFile;
use percent_encoding::percent_decode_str;
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

// we want to return a function that filters elements
fn parse_query(s: &str) -> impl Fn(Vec<f64>) -> bool {
    let s = percent_decode_str(s).decode_utf8().unwrap();
    let mut chars = s.chars().peekable();
    let mut field = None;
    let mut fun: Option<fn(&f64, &f64) -> bool> = None;
    let mut digit = String::new();
    let mut abs = false;
    while let Some(c) = chars.next() {
        match c {
            '|' => abs = true,
            '$' => {
                field = chars.next().unwrap().to_digit(10);
            }
            '>' => {
                if chars.peek().is_some_and(|&c| c == '=') {
                    fun = Some(f64::ge);
                    chars.next().unwrap();
                } else {
                    fun = Some(f64::gt)
                }
            }
            '<' => {
                if chars.peek().is_some_and(|&c| c == '=') {
                    fun = Some(f64::le);
                    chars.next().unwrap();
                } else {
                    fun = Some(f64::lt)
                }
            }
            '0'..='9' | '.' | '-' => digit.push(c),
            _ => todo!("{}", c),
        }
    }
    let digit = digit.parse::<f64>().unwrap();
    move |x: Vec<f64>| {
        let mut v = x[field.unwrap() as usize - 1];
        if abs {
            v = v.abs();
        }
        (fun.unwrap())(&v, &digit)
    }
}

async fn index(data: Datum, req: HttpRequest) -> impl Responder {
    let s = req.query_string();
    let sp: Vec<_> = s.split('=').collect();
    let query = if let Some(s) = sp.get(1) && !s.is_empty() {
        Some(parse_query(s))
    } else {
        None
    };

    use std::fmt::Write;
    let mut body = String::new();
    let records = data.records.read().unwrap();
    writeln!(body, "<table id=\"myTable2\">").unwrap();
    writeln!(body, "<tr>").unwrap();
    writeln!(body, "<th onclick=\"sortTable(0)\">Record ID</th>").unwrap();
    writeln!(body, "<th onclick=\"sortTable(1)\">DDE</th>").unwrap();
    writeln!(body, "</tr>").unwrap();
    for record in records.iter() {
        // either the query is empty or it is some and matches
        if query.as_ref().is_some_and(|q| q(vec![record.dde]))
            || query.as_ref().is_none()
        {
            writeln!(body, "<tr>").unwrap();
            writeln!(
                body,
                "<td><a href=/pic?id={0}>{}</a></td><td>{:.6}</td>",
                record.id, record.dde
            )
            .unwrap();
            writeln!(body, "</tr>").unwrap();
        }
    }
    writeln!(body, "</table>").unwrap();

    let index = load("static/index.html");
    HttpResponse::Ok().body(index.replace("{{body}}", &body))
}

async fn pic(data: Datum, req: HttpRequest) -> impl Responder {
    // let records = data.records.read().unwrap();
    let map = data.map.read().unwrap();
    let query = req.query_string();
    let sp: Vec<_> = query.split('=').collect();
    // TODO cache the generated SVG files
    let mol = Molecule::from_mapped_smiles(&map[sp[1]]).unwrap();
    HttpResponse::Ok().body(mol.to_svg())
}

#[allow(unused)]
struct State {
    records: RwLock<Vec<Record>>,
    map: RwLock<HashMap<String, String>>,
}

impl State {
    fn new(records: Vec<Record>, map: HashMap<String, String>) -> Self {
        Self {
            records: RwLock::new(records),
            map: RwLock::new(map),
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

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let records = load_records(
        "/home/brent/omsf/projects/benchmarking/output/industry/tm/dde.csv",
    );

    let map = load_dataset(
        "/home/brent/omsf/projects/benchmarking/datasets/industry.json",
    );

    let state = web::Data::new(State::new(records, map));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(index))
            .route("/pic", web::get().to(pic))
            .route("/css/{filename:.*}", web::get().to(css_file))
            .route("/js/{filename:.*}", web::get().to(js_file))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
