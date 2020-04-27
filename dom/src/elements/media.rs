//! HTML supports various multimedia resources such as images, audio, and video.

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

html_element! {
    /// The [HTML `<area>` element][mdn] defines a hot-spot region on an image, and optionally
    /// associates it with a [hypertext link]. This element is used only within a [`<map>`][map]
    /// element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/area
    /// [hypertext link]: https://developer.mozilla.org/en-US/docs/Glossary/Hyperlink
    /// [map]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map
    area -> Area
}

impl Area {
    attr_method! {
        /// A set of values specifying the coordinates of the hot-spot region.
        pub coords
    }
}

html_element! {
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

html_element! {
    /// The [HTML `<img>` element][mdn] embeds an image into the document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img
    img -> Image
}

impl Image {
    attr_method! {
        /// Indicates the preferred method to decode the image.
        pub decoding
    }

    attr_method! {
        /// This attribute tells the browser to ignore the actual intrinsic size of the image and
        /// pretend it’s the size specified in the attribute.
        pub intrinsicsize
    }

    attr_method! {
        /// Indicates that the image is part of a server-side image map.
        pub ismap
    }
}

html_element! {
    /// The [HTML `<map>` element][mdn] is used with [`<area>`][area] elements to define an image
    /// map (a clickable link area).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map
    /// [area]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/area
    map -> Map
}

html_element! {
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

impl Track {
    attr_method! {
        /// Indicates that the track should be enabled unless the user's preferences indicate something
        /// different.
        pub default
    }

    attr_method! {
        /// Specifies the kind of text track.
        pub kind
    }

    attr_method! {
        /// Language of the track text data. It must be a valid [BCP 47] language tag. If the kind
        /// attribute is set to subtitles, then srclang must be defined.
        ///
        /// [BCP 47]: https://r12a.github.io/app-subtags/
        pub srclang
    }
}

html_element! {
    /// The [HTML Video element (`<video>`)][mdn] embeds a media player which supports video
    /// playback into the document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video
    video -> Video
}

impl Video {
    attr_method! {
        /// A URL indicating a poster frame to show until the user plays or seeks.
        pub poster
    }
}
