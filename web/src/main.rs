use rocket::{
	catch, catchers,
	fs::{relative, NamedFile},
	get,
	http::CookieJar,
	launch,
	response::Redirect,
};
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
		Path::new(relative!("static")).join(path)
	};

	NamedFile::open(path).await.ok()
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
