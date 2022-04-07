use std::path::Path;
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

					let mimetype = part.content_type();

					let mimetype = if mimetype.is_none() || mimetype.unwrap() == "application/octet-stream" {
						tree_magic::from_u8(&data)
					} else {
						mimetype.unwrap().to_string()
					};

					let parsed = Upload{
						filename,
						mimetype,
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
