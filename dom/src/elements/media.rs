//! HTML supports various multimedia resources such as images, audio, and video.

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

element! {
    /// The [HTML `<area>` element][mdn] defines a hot-spot region on an image, and optionally
    /// associates it with a [hypertext link]. This element is used only within a [`<map>`][map]
    /// element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/area
    /// [hypertext link]: https://developer.mozilla.org/en-US/docs/Glossary/Hyperlink
    /// [map]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map
    area -> Area
}

element! {
    /// The [HTML `<audio>` element][mdn] is used to embed sound content in documents. It may
    /// contain one or more audio sources, represented using the `src` attribute or the
    /// [`<source>`][source] element: the browser will choose the most suitable one. It can also be
    /// the destination for streamed media, using a [`MediaStream`][stream].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio
    /// [source]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source
    /// [stream]: https://developer.mozilla.org/en-US/docs/Web/API/MediaStream
    audio -> Audio
}

element! {
    /// The [HTML `<img>` element][mdn] embeds an image into the document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img
    img -> Image
}

element! {
    /// The [HTML `<map>` element][mdn] is used with [`<area>`][area] elements to define an image
    /// map (a clickable link area).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map
    /// [area]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/area
    map -> Map
}

element! {
    /// The [HTML `<track>` element][mdn] is used as a child of the media elements
    /// [`<audio>`][audio] and [`<video>`][video]. It lets you specify timed text tracks (or
    /// time-based data), for example to automatically handle subtitles. The tracks are formatted in
    /// [WebVTT format][vtt] (`.vtt` files) — Web Video Text Tracks or [Timed Text Markup Language
    /// (TTML)][ttml].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/track
    /// [audio]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio
    /// [video]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video
    /// [vtt]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Video_Text_Tracks_Format
    /// [ttml]: https://w3c.github.io/ttml2/index.html
    track -> Track
}

element! {
    /// The [HTML Video element (`<video>`)][mdn] embeds a media player which supports video
    /// playback into the document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video
    video -> Video
}
