use std::path::{Path, PathBuf};

use rocket;
use rocket::response::NamedFile;
use rocket_contrib::Template;

lazy_static! {
    static ref PATH_STATIC: &'static Path = Path::new("static/");
    static ref CONTEXT: &'static Context = &Context {
        title: "Conway's Game of Life",
        default_grid: r#"
..........
...x......
....x.....
..xxx.....
..........
..........
..........
..........
..........
        "#,
    };
}

#[derive(Serialize)]
struct Context {
    title: &'static str,
    default_grid: &'static str,
}

pub fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![route_index, route_static])
        .attach(Template::fairing())
}

#[get("/", format = "text/html")]
fn route_index() -> Template {
    Template::render("index", *CONTEXT)
}

#[get("/static/<file..>")]
fn route_static(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(PATH_STATIC.join(file)).ok()
}
