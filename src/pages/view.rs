use askama::Template;
use mime::Mime;

#[derive(Template)]
#[template(path = "view.html")]
pub struct ViewPage {
	pub mime_type: Mime,
	pub file_content: String,
}
