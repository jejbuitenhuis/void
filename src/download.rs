use askama::Template;
use mime::Mime;
use crate::{get_upload_path, pages::view::ViewPage};
use std::{
	fs,
	path::Path,
	str::FromStr,
};
use serde::{Serialize, Deserialize};
use warp::{
	Rejection, Reply,
	http::StatusCode, hyper::{Response, Body},
};

use crate::{
	Base64, Database,
};

#[derive(Debug, Deserialize)]
pub struct RetrieveQuery {
	pub v: Option<String>,
	pub view: Option<String>,
}

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

pub fn parse_query_string(query: RetrieveQuery) -> bool {
	query.v.is_some() || query.view.is_some()
}

pub fn get_download_response(file: File) -> Result< Response<Body>, Rejection > {
	let response = Response::builder()
		.status(StatusCode::OK)
		.header("Content-Type", file.mime_type)
		.header( "X-Accel-Redirect", format!("/{}/{}", get_upload_path(), file.filename) )
		.body( Body::empty() )
		.expect("Error unwrapping response");

	Ok(response)
}

pub fn get_view_response(file: File) -> Result< Response<Body>, Rejection > {
	let file_path = Path::new( &get_upload_path() ).join(file.filename);
	let file_content = fs::read_to_string(file_path)
		.map_err(|e| {
			println!("Error reading file for view page: {}", e);

			warp::reject()
		})?;
	let page = ViewPage {
		mime_type: Mime::from_str(&file.mime_type).unwrap(),
		file_content,
	};
	let response = warp::reply::html( page.render().unwrap() );

	Ok( response.into_response() )
}

pub fn get_retrieve_response(file: File, do_view_file: bool) -> Result< Response<Body>, Rejection > {
	if do_view_file {
		get_view_response(file)
	} else {
		get_download_response(file)
	}
}
