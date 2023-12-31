mod upload;

use rocket::{
	catch, catchers,
	fs::{relative, NamedFile},
	get,
	http::CookieJar,
	launch,
	response::Redirect,
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
	let path_string = path.to_str()?.to_owned();
	let last_char = path_string.chars().last()?;

	let static_dir = Path::new(relative!("static"));
	let static_file_path = static_dir.join(&path_string);

	if Path::exists(&static_file_path) {
		return NamedFile::open(static_file_path).await.ok();
	}

	let webshare_dir = Path::new(relative!("static/webshare"));
	let webshare_file_path = webshare_dir.join(&path_string);

	let path = if last_char == '-' {
		let mut webshare_path_string = path_string;
		webshare_path_string.pop();
		let delete_path = webshare_dir.join(&webshare_path_string);
		let secret = env::var("AUTH_COOKIE").expect("Missing environment variable: AUTH_COOKIE");
		let auth_cookie = cookies.get("auth");

		if let Some(auth_cookie) = auth_cookie {
			if auth_cookie
				.value()
				.as_bytes()
				.ct_eq(secret.as_bytes())
				.unwrap_u8() == 1
			{
				let _ = std::fs::remove_file(&delete_path);
			}
		}

		delete_path
	} else {
		webshare_file_path
	};

	NamedFile::open(path).await.ok()
}

#[catch(404)]
pub async fn not_found() -> Result<NamedFile, std::io::Error> {
	NamedFile::open("static/404.html").await
}

#[get("/")]
fn root_redirect() -> Redirect {
	Redirect::permanent("https://elg.gg")
}

#[launch]
fn rocket() -> _ {
	dotenvy::dotenv().ok();
	rocket::build()
		.mount(
			"/",
			rocket::routes![
				root_redirect,
				static_files,
				version,
				upload::endpoint,
				upload::page
			],
		)
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
