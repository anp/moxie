use actix_web::{
    dev::{BodySize, MessageBody, ResponseBody, ServiceResponse},
    error::Error as WebError,
    web,
};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tracing::*;

pub fn reload_on_changes_into_html<B>(
    response: ServiceResponse<B>,
) -> ServiceResponse<XmlRewritingBody<B>>
where
    B: MessageBody,
{
    if let Some(content_type) = response.headers().get("content-type") {
        if content_type != "text/html" {
            return XmlRewritingBody::nop(response);
        }
    }

    let path = response.request().path();
    if path == "/ch-ch-ch-changes" {
        return XmlRewritingBody::nop(response);
    }
    info!({ %path }, "serving html page");

    XmlRewritingBody::nop(response)
}

pub struct XmlRewritingBody<B> {
    inner: ResponseBody<B>,
}

impl<B> XmlRewritingBody<B> {
    pin_utils::unsafe_pinned!(inner: ResponseBody<B>);

    fn nop(response: ServiceResponse<B>) -> ServiceResponse<Self> {
        response.map_body(|_, inner| ResponseBody::Body(Self { inner }))
    }
}

impl<B> MessageBody for XmlRewritingBody<B>
where
    B: MessageBody + Unpin,
{
    fn size(&self) -> BodySize {
        self.inner.size()
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<web::Bytes, WebError>>> {
        <ResponseBody<B> as MessageBody>::poll_next(self.inner(), cx)
    }
}
