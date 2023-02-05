use std::{
	path::Path, fs::File,
	io::{Read, Write},
};
use sha2::{Digest, Sha512};
use mime::Mime;
use futures::{future, StreamExt};
use warp::{
	Buf, Rejection,
	multipart::FormData,
};

use crate::{
	Base64, Database,
	common::get_upload_path
};

const PASSWORD_HASH_LENGTH: usize = 16;

#[derive(Debug)]
pub struct Upload {
	pub filename: String,
	pub extension: Option<String>,
	pub mime_type: Mime,
	pub password: String,
	data: Vec<u8>,
}

fn get_filename_from_data(data: &Vec<u8>) -> String {
	let mut hasher = Sha512::new();

	hasher.update( data.clone() );

	hex::encode( hasher.finalize() )
}

fn get_password_from_filename(filename: &String) -> Result<String, Rejection> {
	if filename.len() < PASSWORD_HASH_LENGTH {
		println!("Filename is too short to generate a hash from!");

		return Err( warp::reject::reject() );
	}

	// TODO: is this the correct size?
	let mut password = String::with_capacity( filename.len() );
	let steps = ( filename.len() as f64 / PASSWORD_HASH_LENGTH as f64 )
		.ceil() as usize;

	for len in 0..steps {
		let start = len * PASSWORD_HASH_LENGTH;
		let end = start + PASSWORD_HASH_LENGTH;
		let hash_substring = &filename[start..end];

		let num = u64::from_str_radix(hash_substring, 16)
			.map_err(|e| {
				println!("Error when parsing file hash: {}", e);
				warp::reject::reject()
			})?;

		password.push_str( Base64::encode(num).as_str() );
	}

	Ok(password)
}

// TODO: make these custom errors?
async fn parse_form_data(form: FormData) -> Result<Upload, Rejection> {
	form.filter_map(|part| {
		let result = match part {
			Ok(p) => Some(p),
			Err(_) => None,
		};

		future::ready(result)
	})
		.map(|mut part| async move {
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
			let password = get_password_from_filename(&filename)?;

			let mime_type = if mime_type.is_none() || mime_type.unwrap() == "application/octet-stream" {
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

			let result = Upload {
				filename,
				extension,
				mime_type,
				password,
				data,
			};

			Ok::<Upload, Rejection>(result)
		})
		.next() // we only want the first file
		.await
		.ok_or_else(|| {
			println!("No file found!");
			warp::reject::reject()
		})?
		.await
}

pub async fn handle_upload(db: Database, form: FormData) -> Result<String, Rejection> {
	let uploaded_file = parse_form_data(form).await?;
	let upload_path = Path::new( &get_upload_path() )
		.join(&uploaded_file.filename);

	File::options()
		.create_new(true)
		.write(true)
		.open(upload_path)
		.map_err(|e| {
			println!("Error when creating file: {}", e);
			warp::reject::reject()
		})?
		.write_all(&uploaded_file.data)
		.map_err(|e| {
			println!("Error when writing to file: {}", e);
			warp::reject::reject()
		})?;

	let inserted_id = db.upload_file(&uploaded_file)
		.await
		.map_err(|e| {
			println!("Error when storing file in database: {}", e);
			warp::reject::reject()
		})?;

	let encoded_id = Base64::encode(inserted_id);
	let request_file = match &uploaded_file.extension {
		Some(ext) => encoded_id + "." + ext,
		None => encoded_id,
	};
	let upload_location = format!(
		"{}?pass={}",
		request_file,
		uploaded_file.password,
	);

	Ok::<_, Rejection>(upload_location)
}
