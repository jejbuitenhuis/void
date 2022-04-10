use serde::Serialize;
use sqlx::{
	MySql, Pool,
	mysql::MySqlPoolOptions,
};

use crate::upload::Upload;

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

	pub async fn upload_file(&self, file: &Upload) -> Result<u64, sqlx::Error> {
		let result = sqlx::query!(
			"INSERT INTO File (filename, extension, mime_type) VALUES (?, ?, ?)",
			file.filename,
			file.extension,
			file.mime_type.to_string(),
		)
			.execute(&self.pool)
			.await?;

		Ok( result.last_insert_id() )
	}
}
