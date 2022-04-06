use std::io::Read;
use warp::Buf;
use crate::Database;
use futures:: StreamExt;
use warp::Rejection;
use warp::Reply;
use warp::multipart::FormData;

#[derive(Debug)]
struct Upload {
	filename: String,
	mimetype: String,
	data: Vec<u8>,
}

pub async fn handle_upload(db: Database, form: FormData) -> Result<impl Reply, Rejection> {
	let parts: Vec<Upload> = form
		.filter_map(|p| async move {
			match p {
				Ok(mut part) => {
					let filename = part.filename().unwrap().to_string();
					let data = part.data()
						.await
						.unwrap()
						.unwrap()
						.reader()
						.bytes()
						.map( |b| b.unwrap() )
						.collect::< Vec<u8> >();

					let mimetype = match dbg!( infer::get( &data ) ) {
						Some(m) => m.mime_type(),
						None => match part.content_type() {
							Some(t) => t,
							None => "text/plain",
						},
					};

					let parsed = Upload{
						filename,
						mimetype: String::from(mimetype),
						data,
					};

					Some(parsed)
				},
				Err(_) => None,
			}
		})
		.collect::< Vec<_> >()
		.await;

	let part = match parts.first() {
		Some(p) => p,
		None => {
			println!("Something went wrong in parts.first()");
			return Err( warp::reject::reject() )
		},
	};

	println!("{:?}", part);

	Ok::<_, Rejection>(
		warp::reply::html("<p>Hi!</p>")
	)
}
