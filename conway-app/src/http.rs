use std::path::{Path, PathBuf};

use rocket;
use rocket::response::NamedFile;

lazy_static! {
    static ref BASE_DIR: &'static Path = Path::new("client/");
}

pub fn server() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![route_index, route_static])
}

#[get("/", format = "text/html")]
fn route_index() -> Option<NamedFile> {
    NamedFile::open(BASE_DIR.join("dist/index.html")).ok()
}

#[get("/static/<file..>")]
fn route_static(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(BASE_DIR.join(file)).ok()
}
