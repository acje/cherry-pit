use std::collections::BTreeMap;
use pardosa_genome::GenomeSafe;
use serde::Serialize;

/// Derive macro detects K in BTreeMap inside an enum variant.
#[derive(GenomeSafe, Serialize)]
enum Container<K> {
    Empty,
    WithMap { entries: BTreeMap<K, u32> },
}

fn main() {
    let _ = Container::<String>::SCHEMA_HASH;
}
