use serde::Serialize;
use std::{collections::HashMap, fmt::Display, fs::File, path::Path};

use openff_toolkit::qcsubmit::results::{Entry, ResultCollection};
/// DDE records loaded from a CSV file
pub struct Record {
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
pub fn load_dataset(filename: impl AsRef<Path>) -> HashMap<String, String> {
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

pub fn load_records(filename: impl AsRef<Path>) -> Vec<Record> {
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

#[derive(Serialize)]
pub(crate) struct Row {
    pub id: String,
    pub vals: Vec<f64>,
    pub show: bool,
}

pub fn build_rows(records: Vec<Vec<Record>>) -> Vec<Row> {
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
            ret.push(Row {
                id,
                vals,
                show: true,
            });
        } else {
            eprintln!(
                "warning: omitting record {id} for {}/{lr} fields",
                vals.len()
            );
        }
    }
    ret
}
