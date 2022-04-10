use askama::Template;
use std::convert::Infallible;
use warp::{
	Filter, Rejection,
	http::StatusCode
};

use crate::{
	base64::Base64,
	database::Database,
	upload::handle_upload,
	pages::home::HomePage,
};

mod base64;
mod database;
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
		let test = warp::path!("test")
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
			.or(upload)
			.or(test)
			.recover(|err| async move {
				eprintln!("Something went wrong, got a rejection: {:?}", err);

				Ok::<_, Infallible>( warp::reply::reply() )
			})
	};

	warp::serve(endpoints)
		.run(([127, 0, 0, 1], 8080)).await;
}
