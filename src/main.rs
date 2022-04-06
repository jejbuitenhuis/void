use askama::Template;
use crate::database::Database;
use crate::pages::home::HomePage;
use std::convert::Infallible;
use serde::Serialize;
use warp::Filter;

mod database;
mod pages;

#[derive(Serialize)]
struct Hello<'a> {
	hello: &'a str,
}

fn with_database(database: Database) -> impl Filter<Extract = (Database,), Error = Infallible> + Clone {
	warp::any().map( move || database.clone() )
}

#[tokio::main]
async fn main() {
	let database_url = dotenv::var("DATABASE_URL").unwrap();
	let database = Database::new(database_url).await;

	let endpoints = {
		let home = warp::get()
			.and( warp::path::end() )
			.and_then(|| async {
				Ok::<_, Infallible>(
					warp::reply::html(
						HomePage{}.render().unwrap()
					)
				)
			});

		home
	};

	warp::serve(endpoints)
		.run( ( [ 127, 0, 0, 1 ], 8080 ) )
		.await;
}
