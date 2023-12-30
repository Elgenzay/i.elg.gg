use rocket::{
	catch, catchers,
	fs::{relative, NamedFile},
	get,
	http::CookieJar,
	launch,
	serde::json::Json,
};
use serde::Serialize;
use std::{
	env,
	path::{Path, PathBuf},
};
use subtle::ConstantTimeEq;

#[get("/<path..>")]
pub async fn static_files(path: PathBuf, cookies: &CookieJar<'_>) -> Option<NamedFile> {
	let mut path_string = path.to_owned().into_os_string().into_string().ok()?;
	let last_char = path_string.chars().last()?;

	let path = if last_char == '-' {
		path_string.pop();
		let path = Path::new(relative!("static")).join(path_string);
		let secret = env::var("AUTH_COOKIE").expect("Missing environment variable: AUTH_COOKIE");

		let auth_cookie = match cookies.get("auth") {
			Some(v) => v,
			None => return None,
		};

		if auth_cookie
			.value()
			.as_bytes()
			.ct_eq(secret.as_bytes())
			.unwrap_u8() == 1
		{
			std::fs::remove_file(&path).unwrap();
		}

		path
	} else {
		Path::new(relative!("static/webshare")).join(path)
	};

	NamedFile::open(path).await.ok()
}

#[catch(404)]
pub async fn not_found() -> Result<NamedFile, std::io::Error> {
	NamedFile::open("static/404.html").await
}

#[get("/favicon.ico")]
async fn favicon() -> Option<NamedFile> {
	NamedFile::open(Path::new("static/favicon.ico")).await.ok()
}

#[launch]
fn rocket() -> _ {
	dotenvy::dotenv().ok();
	rocket::build()
		.mount("/", rocket::routes![static_files, version, favicon])
		.register("/", catchers![not_found])
}

#[derive(Serialize)]
pub struct VersionInfo {
	version: String,
}

#[rocket::get("/version")]
pub fn version() -> Json<VersionInfo> {
	Json(VersionInfo {
		version: env!("CARGO_PKG_VERSION").to_string(),
	})
}
