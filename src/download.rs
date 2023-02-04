use std::path::Path;
use serde::Serialize;
use warp::Rejection;
use crate::{
	Base64, Database,
};

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct File {
	pub filename: String,
	pub extension: Option<String>,
	pub mime_type: String,
}

pub async fn get_file(db: Database, requested_file: String) -> Result<File, Rejection> {
	let path = Path::new(&requested_file);
	let filename = match path.file_stem() {
		Some(f) => f.to_str().expect("Error unwrapping OsStr to str").to_string(),
		None => {
			println!("Error while extracting filename");
			return Err( warp::reject::reject() );
		}
	};
	let file_id = match Base64::decode(filename) {
		Ok(i) => i,
		Err(e) => {
			println!("Error while extracting id from filename: {}", e);
			return Err( warp::reject::reject() );
		}
	};
	let file = match db.get_file_information(file_id).await {
		Ok(f) => f,
		Err(e) => {
			println!("Error while fetching hash: {}", e);
			return Err( warp::reject::reject() );
		}
	};
	let file = match file {
		Some(f) => f,
		None => {
			println!("No file found with the name \"{}\" (id: {})", requested_file, file_id);
			return Err( warp::reject::reject() );
		}
	};

	Ok(file)
}
