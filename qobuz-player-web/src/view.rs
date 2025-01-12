use axum::response::IntoResponse;
use leptos::IntoView;

#[macro_export]
macro_rules! html {
    ($($body:tt)*) => {
        leptos::view!($($body)*)
    };
}

pub fn render(view: impl IntoView) -> axum::response::Response {
    (
        [
            (
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
            ),
            (
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("no-cache"),
            ),
        ],
        view.to_html(),
    )
        .into_response()
}
