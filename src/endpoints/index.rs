use axum::http::HeaderMap;
use axum::response::Html;
use sailfish::TemplateOnce;
use crate::endpoints::common::*;
use crate::error::AppError;

#[derive(TemplateOnce)]
#[template(path = "index.stpl")]
struct IndexPageTemplate {
    auth_type: AuthUserType
}

pub async fn index(
    headers: HeaderMap,
) -> Result<Html<String>, AppError> {



    let ctx = IndexPageTemplate {
        auth_type: get_auth_type_from_headers(&headers)
    };

    let body = ctx.render_once().map_err(|e| AppError(e.into()))?;
    Ok(Html::from(body))
}