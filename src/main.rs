use warp::Filter;
use crate::upload::handle_upload;
use crate::database::Database;
use crate::pages::home::HomePage;
use askama::Template;
use std::convert::Infallible;

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
		.unwrap_or("536870912".to_string()) // 512 MiB
		.parse::<u64>()
		.unwrap();
	let database_url = dotenv::var("DATABASE_URL").unwrap();
	let database = Database::new(database_url).await;

	let endpoints = {
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
			.and_then(handle_upload);

		home.or(upload)
	};

	warp::serve(endpoints)
		.run(([127, 0, 0, 1], 8080)).await;
}
