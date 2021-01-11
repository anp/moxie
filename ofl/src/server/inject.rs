use actix_web::{
    dev::{MessageBody, ResponseBody, ServiceResponse},
    error::Error as WebError,
    web,
};
use futures::stream::{Stream, StreamExt};
use lol_html::{element, html_content::ContentType, RewriteStrSettings};
use tracing::*;

const RELOAD_ON_CHANGES: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/reloadOnChanges.js"));
const CHANGES_URL: &str = "/ch-ch-ch-changes";

pub async fn reload_on_changes_into_html<B>(
    mut response: ServiceResponse<B>,
) -> Result<ServiceResponse<B>, actix_web::error::Error>
where
    B: MessageBody + Unpin,
{
    if let Some(content_type) = response.headers().get("content-type") {
        if content_type != "text/html" {
            return Ok(response);
        }
    }

    let path = response.request().path();
    if path == CHANGES_URL {
        return Ok(response);
    }
    info!({ %path }, "serving html page");

    let mut body = collect_body(response.take_body()).await;

    if let Some(rewritten) = inject_script_tag(&body) {
        body = rewritten;
    }

    Ok(response.map_body(|_, _| ResponseBody::Other(body.into())))
}

fn inject_script_tag(body: &str) -> Option<String> {
    lol_html::rewrite_str(
        &body,
        RewriteStrSettings {
            element_content_handlers: vec![element!("head", |head| {
                info!("inserting script tag");
                head.append("<script>", ContentType::Html);
                head.append(RELOAD_ON_CHANGES, ContentType::Html);
                head.append("</script>", ContentType::Html);
                Ok(())
            })],
            ..Default::default()
        },
    )
    .ok()
}

async fn collect_body(
    mut body: impl Stream<Item = Result<web::Bytes, WebError>> + Unpin,
) -> String {
    let mut buf = Vec::new();

    while let Some(Ok(bytes)) = body.next().await {
        buf.extend(bytes);
    }

    String::from_utf8(buf).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_injected {
        ($s:expr) => {{
            let base = $s;
            let injected = inject_script_tag(base).unwrap();
            assert!(!base.contains(CHANGES_URL), "original shouldn't have reload URL");
            assert!(injected.contains(CHANGES_URL), "must have injected reload URL");
            assert!(injected.contains("=>"), "must preserve lambda notation without escaping")
        }};
    }

    macro_rules! assert_untouched {
        ($s:expr) => {{
            let base = $s;
            assert_eq!(inject_script_tag(base).unwrap(), base, "contents shouldn't change");
        }};
    }

    #[test]
    fn inject() {
        assert_injected!(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../index.html")));
    }

    #[test]
    fn inject_weird_whitespace() {
        assert_injected!("<html> <head> </ head > </html>");
    }

    #[test]
    fn inject_toml_nop() {
        assert_untouched!(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml")));
    }

    #[test]
    fn inject_parse_fail_nop() {
        assert_untouched!("<html> <head </html>");
    }
}
