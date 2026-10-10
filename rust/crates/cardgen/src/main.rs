use cardgen::data::Project;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("cardgen: {e}");
        std::process::exit(1);
    }
}

fn flag(args: &mut Vec<String>, name: &str) -> bool {
    match args.iter().position(|a| a == name) {
        Some(i) => {
            args.remove(i);
            true
        }
        None => false,
    }
}

fn opt(args: &mut Vec<String>, name: &str) -> Result<Option<String>, String> {
    match args.iter().position(|a| a == name) {
        Some(i) => {
            args.remove(i);
            if i < args.len() { Ok(Some(args.remove(i))) } else { Err(format!("{name} needs a value")) }
        }
        None => Ok(None),
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let mut args = args.to_vec();
    let root = opt(&mut args, "--root")?;
    let project = match root {
        Some(r) => Project::new(r),
        None => Project::find()?,
    };
    let cmd = if args.is_empty() { String::new() } else { args.remove(0) };
    match cmd.as_str() {
        "export" => {
            let offline = flag(&mut args, "--offline");
            let out = opt(&mut args, "--out")?.map(PathBuf::from).unwrap_or_else(|| project.root.join("data"));
            let (c, d) = cardgen::export::run(&project, &out, offline)?;
            println!("{c} cards, {d} decks -> {}", out.display());
            Ok(())
        }
        _ => Err("usage: cardgen export [--out DIR] [--offline] | audit | deck ... (see crates/cardgen/README.md)".into()),
    }
}
