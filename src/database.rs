use serde::Serialize;
use sqlx::{
	MySql, Pool,
	mysql::MySqlPoolOptions,
};

use crate::{
	download::File,
	upload::Upload,
};

#[derive(Clone)]
pub struct Database {
	pool: Pool<MySql>,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct Test {
	pub id: u32,
	pub text: String,
}

impl Database {
	pub async fn new(url: String) -> Database {
		let pool = MySqlPoolOptions::new()
			.max_connections(5)
			.connect( url.as_str() )
			.await
			.expect("Failed to connect to the database");

		Database { pool }
	}

	pub async fn get_file_information(&self, file_id: u64) -> Result< Option<File>, sqlx::Error > {
		let result = sqlx::query_as!(
			File,
			"SELECT filename, extension, mime_type, password FROM File WHERE id = ?",
			file_id
		)
			.fetch_optional(&self.pool)
			.await?;

		Ok(result)
	}

	pub async fn upload_file(&self, file: &Upload) -> Result<u64, sqlx::Error> {
		let result = sqlx::query!(
			"INSERT INTO File (filename, extension, mime_type, password) VALUES (?, ?, ?, ?)",
			file.filename,
			file.extension,
			file.mime_type.to_string(),
			file.password,
		)
			.execute(&self.pool)
			.await?;

		Ok( result.last_insert_id() )
	}
}
