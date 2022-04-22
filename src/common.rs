/// Get the path to the folder where the uploaded files should be stored. This
/// path does **not** contain a trailing slash (`/`).
pub fn get_upload_path() -> String {
	dotenv::var("UPLOAD_FOLDER")
		.unwrap_or( "uploaded_files".to_string() )
		.trim_end_matches('/')
		.to_string()
}
