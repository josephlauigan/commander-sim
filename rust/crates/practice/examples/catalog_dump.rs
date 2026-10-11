//! Print the setup screen's catalog as JSON (to compare with Python's `catalog.catalog()`):
//!     cargo run --release -p practice --example catalog_dump -- data
fn main() {
    let data = std::env::args().nth(1).unwrap_or_else(|| "data".into());
    match practice::catalog::catalog(std::path::Path::new(&data)) {
        Ok(c) => println!("{c}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
