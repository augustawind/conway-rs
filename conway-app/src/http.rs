use std::path::{Path, PathBuf};

use rocket;
use rocket::response::NamedFile;

lazy_static! {
    static ref PATH_STATIC: &'static Path = Path::new("static/");
    static ref CONTEXT: &'static Context = &Context {
        title: "Conway's Game of Life",
        default_grid: include_str!("../static/patterns/default"),
    };
}

#[derive(Serialize)]
struct Context {
    title: &'static str,
    default_grid: &'static str,
}

pub fn server() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![route_index, route_static])
}

#[get("/", format = "text/html")]
fn route_index() -> Option<NamedFile> {
    NamedFile::open(PATH_STATIC.join("index.html")).ok()
}

#[get("/static/<file..>")]
fn route_static(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(PATH_STATIC.join(file)).ok()
}
