//! First-launch data copy for the iPad app (a port of the old Briefcase app's `bootstrap.py::sync_data`).
//!
//! An iOS app's bundle is read-only, so the data the bundle carries (`<bundle>/data`, `<bundle>/decklists`) is copied
//! into the app's own writable folder (`<app>/data`, `<app>/decklists`; the catalog reads the deck lists next to the
//! data folder). The copy happens when the bundle's `data/bundle.json` stamp differs from the one in `<app>/data`:
//!   - plain files in `data/` are replaced (a new build's card data must win);
//!   - `data/images/` (card images downloaded since) only gets the files it is missing, and its `index.json` is merged;
//!   - other folders (`printings/`, `decklists/`) are replaced file by file. Python copied only the missing files
//!     there, but these are part of a build, not something downloaded: a new build's printings must win;
//!   - saved games live in `<app>/saves`, outside `data/`, and are never touched.

use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;

pub const STAMP: &str = "bundle.json";
const IMAGES: &str = "images";
const INDEX: &str = "index.json";

fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

/// write through a temp file so a kill mid-copy never leaves half a file
fn copy_file(src: &Path, dst: &Path) -> io::Result<()> {
    let tmp = dst.with_extension("tmp-copy");
    fs::copy(src, &tmp)?;
    fs::rename(tmp, dst)
}

/// copy `src`'s files into `dst`: replacing them, or (`only_missing`) only those `dst` lacks
fn copy_dir(src: &Path, dst: &Path, only_missing: bool) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        let (from, to) = (e.path(), dst.join(e.file_name()));
        if from.is_dir() {
            copy_dir(&from, &to, only_missing)?;
        } else if only_missing && e.file_name() == INDEX {
            // the images index: the bundle's entries win, entries for images downloaded since stay
            let mut idx = read_json(&to).filter(Value::is_object).unwrap_or_else(|| Value::Object(Default::default()));
            if let (Some(new), Some(m)) = (read_json(&from).and_then(|v| v.as_object().cloned()), idx.as_object_mut()) {
                m.extend(new);
            }
            let tmp = to.with_extension("tmp-copy");
            fs::write(&tmp, serde_json::to_vec(&idx).unwrap_or_default())?;
            fs::rename(tmp, &to)?;
        } else if !only_missing || !to.exists() {
            copy_file(&from, &to)?;
        }
    }
    Ok(())
}

/// Copy the bundle's `data/` and `decklists/` into the app folder when this build's stamp differs from the last one
/// copied. Returns whether it copied.
pub fn sync_data(bundle: &Path, app: &Path) -> io::Result<bool> {
    let (bdata, data) = (bundle.join("data"), app.join("data"));
    fs::create_dir_all(&data)?;
    let stamp = read_json(&bdata.join(STAMP));
    if stamp.is_some() && read_json(&data.join(STAMP)) == stamp {
        return Ok(false);
    }
    for e in fs::read_dir(&bdata)? {
        let e = e?;
        let (name, from) = (e.file_name(), e.path());
        if name == STAMP {
            continue;
        }
        if from.is_dir() {
            copy_dir(&from, &data.join(&name), name == IMAGES)?;
        } else {
            copy_file(&from, &data.join(&name))?;
        }
    }
    if bundle.join("decklists").is_dir() {
        copy_dir(&bundle.join("decklists"), &app.join("decklists"), false)?;
    }
    if let Some(s) = stamp {
        // last, so a copy that was cut short is redone at the next launch
        fs::write(data.join(STAMP), serde_json::to_vec(&s).unwrap_or_default())?;
    }
    Ok(true)
}
