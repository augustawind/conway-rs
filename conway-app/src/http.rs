use std::path::{Path, PathBuf};

use rocket;
use rocket::response::NamedFile;

lazy_static! {
    static ref DIST_DIR: &'static Path = Path::new("client/dist/");
    static ref VENDOR_DIR: &'static Path = Path::new("client/vendor/");
}

pub fn server() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![route_index, route_static, route_vendor])
}

#[get("/", format = "text/html")]
fn route_index() -> Option<NamedFile> {
    NamedFile::open(DIST_DIR.join("index.html")).ok()
}

#[get("/static/<file..>")]
fn route_static(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(DIST_DIR.join(file)).ok()
}

#[get("/vendor/<file..>")]
fn route_vendor(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(VENDOR_DIR.join(file)).ok()
}
