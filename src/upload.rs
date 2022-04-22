use std::{
	path::Path, fs::File,
	io::{Read, Write},
};
use sha2::{Digest, Sha512};
use mime::Mime;
use futures::StreamExt;
use warp::{
	Buf, Rejection,
	multipart::FormData,
};

use crate::{
	Base64, Database,
	common::get_upload_path
};

#[derive(Debug)]
pub struct Upload {
	pub filename: String,
	pub extension: Option<String>,
	pub mime_type: Mime,
	data: Vec<u8>,
}

fn get_filename_from_data(data: &Vec<u8>) -> String {
	let mut hasher = Sha512::new();

	hasher.update( data.clone() );

	hex::encode( hasher.finalize() )
}

async fn parse_form_data(form: FormData) -> Vec<Upload> {
	form.filter_map(|p| async move {
		match p {
			Ok(mut part) => {
				let data = part.data()
					.await
					.unwrap()
					.unwrap()
					.reader()
					.bytes()
					.map( |b| b.unwrap() )
					.collect::< Vec<u8> >();
				let filename = get_filename_from_data(&data);
				let mime_type = part.content_type();
				let extension = Path::new( part.filename().unwrap_or("") )
					.extension()
					// default to empty extension when conversion fails
					.map( |e| e.to_str().unwrap_or("").to_string() );

				let mime_type: Mime = if mime_type.is_none() || mime_type.unwrap() == "application/octet-stream" {
					match tree_magic::from_u8(&data).parse() {
						Ok(t) => t,
						Err(_) => mime::TEXT_PLAIN,
					}
				} else {
					match mime_type.unwrap().parse() {
						Ok(t) => t,
						Err(_) => mime::TEXT_PLAIN,
					}
				};

				let parsed = Upload{
					filename,
					extension,
					mime_type,
					data,
				};

				Some(parsed)
			},
			Err(_) => None,
		}
	})
	.collect::< Vec<_> >()
	.await
}

pub async fn handle_upload(db: Database, form: FormData) -> Result<String, Rejection> {
	let uploaded_files = parse_form_data(form).await;

	let uploaded_file = match uploaded_files.first() {
		Some(p) => p,
		None => {
			println!("Something went wrong in parts.first()");
			return Err( warp::reject::reject() )
		},
	};

	let upload_path = Path::new( &get_upload_path() ).join(&uploaded_file.filename);

	let file = File::options()
		.create_new(true)
		.write(true)
		.open(upload_path);

	let mut file = match file {
		Ok(f) => f,
		Err(e) => {
			println!("Error when creating file: {}", e);
			return Err( warp::reject::reject() );
		}
	};

	match file.write_all(&uploaded_file.data) {
		Err(e) => {
			println!("Error when writing to file: {}", e);
			return Err( warp::reject::reject() );
		},
		_ => (),
	};

	let inserted_id = match db.upload_file(uploaded_file).await {
		Ok(id) => id,
		Err(e) => {
			println!("Error when storing file in database: {}", e);
			return Err( warp::reject::reject() );
		},
	};

	let encoded_id = Base64::encode(inserted_id);
	let request_file = match &uploaded_file.extension {
		Some(ext) => encoded_id + "." + ext,
		None => encoded_id,
	};

	Ok::<_, Rejection>(request_file)
}
