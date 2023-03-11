#[macro_use]
extern crate rocket;

use rocket::{
	fs::{relative, NamedFile},
	response::Redirect,
};

use std::{
	env,
	path::{Path, PathBuf},
};

#[get("/<path..>")]
pub async fn static_files(path: PathBuf) -> Option<NamedFile> {
	NamedFile::open(Path::new(relative!("static")).join(path))
		.await
		.ok()
}

#[catch(404)]
pub async fn not_found(req: &rocket::Request<'_>) -> Redirect {
	let fallback =
		env::var("FALLBACK_DOMAIN").expect("Missing environment variable: FALLBACK_DOMAIN");
	let mut new_uri = format!("https://{}", fallback);
	let path = PathBuf::from(req.uri().path().to_string())
		.into_os_string()
		.into_string()
		.unwrap();
	new_uri.push_str(&path[..]);
	Redirect::to(new_uri)
}

#[launch]
fn rocket() -> _ {
	dotenvy::dotenv().ok();
	rocket::build()
		.mount("/", rocket::routes![static_files])
		.register("/", catchers![not_found])
}
