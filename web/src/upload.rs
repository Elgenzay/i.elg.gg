use rand::seq::IteratorRandom;
use rocket::data::{Data, ToByteUnit};
use rocket::fs::NamedFile;
use rocket::http::CookieJar;
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest};
use rocket::response::status;
use rocket::Request;
use rocket::{fs::relative, get, post};
use std::env;
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;

const FILENAME_CHARSET: &str = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz123456789";
const FILENAME_LENGTH: usize = 6;
const FILE_SIZE_LIMIT_MB: u64 = 10;

pub struct FileExtension(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for FileExtension {
	type Error = ();

	async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
		match request.headers().get_one("File-Extension") {
			Some(value) => Outcome::Success(FileExtension(value.to_string())),
			None => Outcome::Error((Status::BadRequest, ())),
		}
	}
}

#[post("/upload", data = "<file_data>")]
pub async fn endpoint<'r>(
	cookies: &CookieJar<'_>,
	file_data: Data<'_>,
	file_extension: FileExtension,
) -> Result<String, status::Custom<String>> {
	let secret = env::var("AUTH_COOKIE").expect("Missing environment variable: AUTH_COOKIE");

	let auth_cookie = match cookies.get("auth") {
		Some(cookie) => cookie,
		None => {
			return Err(status::Custom(
				Status::Unauthorized,
				"Auth missing".to_string(),
			));
		}
	};

	if auth_cookie
		.value()
		.as_bytes()
		.ct_eq(secret.as_bytes())
		.unwrap_u8()
		!= 1
	{
		return Err(status::Custom(
			Status::Unauthorized,
			"Auth invalid".to_string(),
		));
	}

	let filename = format!(
		"{}.{}",
		random_string(FILENAME_CHARSET, FILENAME_LENGTH),
		file_extension.0
	);

	let path = PathBuf::from(relative!("static/webshare")).join(&filename);

	file_data
		.open(FILE_SIZE_LIMIT_MB.megabytes())
		.into_file(path)
		.await
		.map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

	Ok(filename)
}

fn random_string(charset: &str, length: usize) -> String {
	let mut rng = rand::thread_rng();

	std::iter::repeat(())
		.map(|()| charset.chars().choose(&mut rng).unwrap())
		.take(length)
		.collect()
}

#[get("/p")]
pub async fn page() -> Option<NamedFile> {
	NamedFile::open(Path::new("static/upload.html")).await.ok()
}
