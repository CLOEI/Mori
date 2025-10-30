use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

pub async fn index() -> IndexTemplate {
    IndexTemplate {}
}
