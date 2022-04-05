use std::convert::Infallible;
use serde::Serialize;
use warp::Filter;

#[derive(Serialize)]
struct Hello<'a> {
	hello: &'a str,
}

#[tokio::main]
async fn main() {
	let endpoints = {
		let hello = warp::get()
			.and( warp::path::end() )
			.and_then(|| async {
				let obj = Hello{ hello: "world" };

				Ok::<_, Infallible>(
					warp::reply::json(&obj)
				)
			});

		hello
	};

	warp::serve(endpoints)
		.run( ( [ 127, 0, 0, 1 ], 8080 ) )
		.await;
}
