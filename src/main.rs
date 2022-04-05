use crate::database::Database;
use std::convert::Infallible;
use serde::Serialize;
use warp::Filter;

mod database;

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
		let hello = warp::get()
			.and( warp::path::end() )
			.and_then(|| async {
				let obj = Hello{ hello: "world" };

				Ok::<_, Infallible>(
					warp::reply::json(&obj)
				)
			});

		let test = warp::get()
			.and( with_database( database.clone() ) )
			.and( warp::path::path("test") )
			.and( warp::path::end() )
			.and_then(|db: Database| async move {
				let result = db.get_test() .await
					.expect("Something went wrong while getting the test rows");

				Ok::<_, Infallible>(
					warp::reply::json(&result)
				)
			});

		hello
			.or(test)
	};

	warp::serve(endpoints)
		.run( ( [ 127, 0, 0, 1 ], 8080 ) )
		.await;
}
