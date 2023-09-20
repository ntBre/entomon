//! web ui for debugging torsion multiplicity issues

use std::{collections::HashMap, fmt::Display, fs::File, path::Path};

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

fn main() {
    let records = load_records(
        "/home/brent/omsf/projects/benchmarking/output/industry/tm/dde.csv",
    );

    let map = load_dataset(
        "/home/brent/omsf/projects/benchmarking/datasets/industry.json",
    );

    for record in &records[..5] {
        let mol = Molecule::from_mapped_smiles(&map[&record.id]).unwrap();
        println!("{} => {}", record.id, map[&record.id]);
        dbg!(mol.to_svg());
    }
}
