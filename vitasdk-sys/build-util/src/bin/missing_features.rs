use std::process::ExitCode;

use build_util::vita_headers_db::VitaDb;

fn main() -> ExitCode {
    const VITA_HEADERS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../vita-headers/db");
    let mut db = VitaDb::load(VITA_HEADERS_PATH.as_ref()).unwrap();
    db.remove_conflicting();

    let mut missing_features = db.missing_features();
    if missing_features.is_empty() {
        ExitCode::SUCCESS
    } else {
        missing_features.sort_unstable();
        missing_features
            .iter()
            .for_each(|feature| println!("{feature} = []"));
        ExitCode::FAILURE
    }
}
