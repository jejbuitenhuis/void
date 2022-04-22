use crate::common::get_upload_path;
use askama::Template;
use std::{convert::Infallible, path::Path, fs::File};
use warp::{
	Filter, Rejection,
	http::StatusCode, hyper::Response
};

use crate::{
	base64::Base64,
	database::Database,
	download::get_file,
	upload::handle_upload,
	pages::home::HomePage,
};

mod base64;
mod common;
mod database;
mod download;
mod upload;
mod pages;

fn with_database(
	database: Database,
) -> impl Filter<Extract = (Database,), Error = Infallible> + Clone {
	warp::any().map(move || database.clone())
}

#[tokio::main]
async fn main() {
	let max_upload_size = dotenv::var("MAX_UPLOAD_SIZE")
		.unwrap_or( "536870912".to_string() ) // 512 MiB
		.parse::<u64>()
		.unwrap();
	let database_url = dotenv::var("DATABASE_URL").unwrap();
	let database = Database::new(database_url).await;

	let endpoints = {
		let test = warp::get()
			.and( warp::path::path("test") )
			.and( warp::path::end() )
			.and_then(|| async {
				let encoded = Base64::encode(1234567890);
				let decoded = Base64::decode( "SLglJB".to_string() ).unwrap();

				Ok::<_, Infallible>(
					warp::reply::html(
						format!("<p>{}</p><p>{}</p>", encoded, decoded)
					)
				)
			});

		let home = warp::get()
			.and( warp::path::end() )
			.and_then(|| async {
				Ok::<_, Infallible>(
					warp::reply::html(
						HomePage {}.render().unwrap()
					)
				)
			});

		let retrieve = warp::get()
			.and( with_database( database.clone() ) )
			.and( warp::path::param() ) // file name
			.and( warp::path::end() )
			.and_then(|db, requested_file: String| async {
				let file = get_file(db, dbg!(requested_file)).await?;
				dbg!(&file);

				let response = Response::builder()
					.status(StatusCode::OK)
					.header("Content-Type", file.mime_type)
					.header( "X-Accel-Redirect", format!("/{}/{}", get_upload_path(), file.filename) )
					.body("")
					.expect("Error unwrapping response");

				Ok::<_, Rejection>(dbg!(response))
			});

		let upload = warp::post()
			.and( with_database( database.clone() ) )
			.and( warp::path::end() )
			.and( warp::multipart::form().max_length(max_upload_size) )
			.and_then(|db, form| async {
				let uploaded_location = handle_upload(db, form).await?;

				Ok::<_, Rejection>(
					warp::reply::with_status(uploaded_location, StatusCode::OK)
				)
			});

		home
			.or(retrieve)
			.or(upload)
			.or(test)
			.recover(|err: Rejection| async move {
				eprintln!("Something went wrong, got a rejection: {:?}", err);

				Ok::<_, Infallible>( warp::reply::html("<p>Something went wrong :(</p>") )
			})
	};

	warp::serve(endpoints)
		.run(([127, 0, 0, 1], 8080)).await;
}
