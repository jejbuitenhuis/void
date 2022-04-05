use serde::Serialize;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySql;
use sqlx::Pool;

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
			.await.expect("Failed to connect to the database");

		Database {
			pool,
		}
	}

	pub async fn get_test(&self) -> Result< Vec<Test>, sqlx::Error > {
		sqlx::query_as!(
			Test,
			"SELECT * FROM Test"
		)
			.fetch_all(&self.pool)
			.await
	}
}
