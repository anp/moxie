//! Types related to security policy.

/// Indicates which referrer to send when fetching the frame's resource.
pub enum ReferrerPolicy {
    /// The Referer header will not be sent.
    NoReferrer,
    /// The Referer header will not be sent to origins without TLS (HTTPS).
    NoReferrerWhenDowngrade,
    /// The sent referrer will be limited to the origin of the referring page:
    /// its scheme, host, and port.
    Origin,
    /// The referrer sent to other origins will be limited to the
    ///   scheme, the host, and the port. Navigations on the same origin will
    /// still include the   path.
    OriginWhenCrossOrigin,
    /// A referrer will be sent for same origin, but cross-origin
    ///   requests will contain no referrer information.
    SameOrigin,
    /// Only send the origin of the document as the referrer
    ///   when the protocol security level stays the same (HTTPS→HTTPS), but
    ///   don't send it to a less secure destination (HTTPS→HTTP).
    StrictOrigin,
    /// Send a full URL when performing a
    ///   same-origin request, only send the origin when the protocol security
    ///   level stays the same (HTTPS→HTTPS), and send no header to a less
    ///   secure destination (HTTPS→HTTP).
    StrictOriginWhenCrossOrigin,
}

impl ReferrerPolicy {
    fn as_str(&self) -> &'static str {
        use ReferrerPolicy::*;
        match self {
            NoReferrer => "no-referrer",
            NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
            Origin => "origin",
            OriginWhenCrossOrigin => "origin-when-cross-origin",
            SameOrigin => "same-origin",
            StrictOrigin => "strict-origin",
            StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
        }
    }
}

impl std::fmt::Display for ReferrerPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
