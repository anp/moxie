//! HTML supports various multimedia resources such as images, audio, and video.

html_element! {
    /// The [HTML `<area>` element][mdn] defines a hot-spot region on an image, and optionally
    /// associates it with a [hypertext link]. This element is used only within a [`<map>`][map]
    /// element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/area
    /// [hypertext link]: https://developer.mozilla.org/en-US/docs/Glossary/Hyperlink
    /// [map]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map
    <area>

    categories {
        Flow, Phrasing
    }

    attributes {
        /// A text string alternative to display on browsers that do not display images. The text
        /// should be phrased so that it presents the user with the same kind of choice as the image
        /// would offer when displayed without the alternative text. This attribute is required only
        /// if the href attribute is used.
        alt

        /// A set of values specifying the coordinates of the hot-spot region. The number and
        /// meaning of the values depend upon the value specified for the shape attribute.
        ///
        /// * rect or rectangle: the coords value is two x,y pairs: left, top, right, bottom.
        /// * circle: the value is x,y,r where x,y is a pair specifying the center of the circle and
        ///   r is a value for the radius.
        /// * poly or polygon: the value is a set of x,y pairs for each point in the polygon:
        ///   x1,y1,x2,y2,x3,y3, and so on.
        ///
        /// The values are numbers of CSS pixels.
        coords

        /// This attribute, if present, indicates that the author intends the hyperlink to be used
        /// for downloading a resource. See <a> for a full description of the download attribute.
        download(bool)

        /// The hyperlink target for the area. Its value is a valid URL. This attribute may be
        /// omitted; if so, the area element does not represent a hyperlink.
        href

        /// Indicates the language of the linked resource. Allowed values are determined by BCP47.
        /// Use this attribute only if the href attribute is present.
        hreflang

        /// Contains a space-separated list of URLs to which, when the hyperlink is followed, POST
        /// requests with the body PING will be sent by the browser (in the background). Typically
        /// used for tracking.
        ping

        /// For anchors containing the href attribute, this attribute specifies the relationship of
        /// the target object to the link object. The value is a space-separated list of link types
        /// values. The values and their semantics will be registered by some authority that might
        /// have meaning to the document author. The default relationship, if no other is given, is
        /// void. Use this attribute only if the href attribute is present.
        rel

        /// This attribute specifies where to display the linked resource. It is a name of, or
        /// keyword for, a browsing context (for example, tab, window, or inline frame). The
        /// following keywords have special meanings:
        ///
        /// * _self: Load the response into the same browsing context as the current one. This value
        ///   is the default if the attribute is not specified.
        /// * _blank: Load the response into a new unnamed browsing context.
        /// * _parent: Load the response into the parent browsing context of the current one. If
        ///   there is no parent, this option behaves the same way as _self.
        /// * _top: Load the response into the top-level browsing context (that is, the browsing
        ///   context that is an ancestor of the current one, and has no parent). If there is no
        ///   parent, this option behaves the same way as _self.
        ///
        /// Use this attribute only if the `href` attribute is present.
        target
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
    <audio>

    categories {
        Flow, Phrasing, Embedded,
        // if it has a `controls` attribute:
        Interactive, Palpable
    }

    children {
        tags {
            <track>,
            <source> // if the element doesn't have a src attribute
        }
    }

    attributes {
        /// If specified, the audio will automatically begin playback as soon as it can do so,
        /// without waiting for the entire audio file to finish downloading.
        ///
        /// Note: Sites that automatically play audio (or videos with an audio track) can be an
        /// unpleasant experience for users, so should be avoided when possible. If you must offer
        /// autoplay functionality, you should make it opt-in (requiring a user to specifically
        /// enable it). However, this can be useful when creating media elements whose source will
        /// be set at a later time, under user control. See our autoplay guide for additional
        /// information about how to properly use autoplay.
        autoplay(bool)

        /// If this attribute is present, the browser will offer controls to allow the user to
        /// control audio playback, including volume, seeking, and pause/resume playback.
        controls(bool)

        /// This enumerated attribute indicates whether to use CORS to fetch the related audio file.
        /// CORS-enabled resources can be reused in the <canvas> element without being tainted.
        ///
        /// When not present, the resource is fetched without a CORS request (i.e. without sending
        /// the Origin: HTTP header), preventing its non-tainted used in <canvas> elements. If
        /// invalid, it is handled as if the enumerated keyword anonymous was used.
        ///
        /// The allowed values are:
        ///
        /// # `anonymous`
        ///
        /// Sends a cross-origin request without a credential. In other words, it sends the
        /// `Origin: HTTP` header without a cookie, X.509 certificate, or performing HTTP Basic
        /// authentication. If the server does not give credentials to the origin site (by not
        /// setting the `Access-Control-Allow-Origin: HTTP` header), the image will be tainted, and
        /// its usage restricted.
        ///
        /// # `use-credentials`
        ///
        /// Sends a cross-origin request with a credential. In other words, it sends the
        /// `Origin: HTTP` header with a cookie, a certificate, or performing HTTP Basic
        /// authentication. If the server does not give credentials to the origin site (through
        /// `Access-Control-Allow-Credentials: HTTP` header), the image will be tainted and its
        /// usage restricted.
        crossorigin


        /// Reading currentTime returns a double-precision floating-point value indicating the
        /// current playback position, in seconds, of the audio. If the audio's metadata isn't
        /// available yet—thereby preventing you from knowing the media's start time or
        /// duration—currentTime instead indicates, and can be used to change, the time at which
        /// playback will begin. Otherwise, setting currentTime sets the current playback position
        /// to the given time and seeks the media to that position if the media is currently loaded.
        ///
        /// If the audio is being streamed, it's possible that the user agent may not be able to
        /// obtain some parts of the resource if that data has expired from the media buffer. Other
        /// audio may have a media timeline that doesn't start at 0 seconds, so setting currentTime
        /// to a time before that would fail. For example, if the audio's media timeline starts at
        /// 12 hours, setting currentTime to 3600 would be an attempt to set the current playback
        /// position well before the beginning of the media, and would fail. The getStartDate()
        /// method can be used to determine the beginning point of the media timeline's reference
        /// frame.
        current_time

        // A double-precision floating-point value which indicates the duration (total length) of
        // the audio in seconds, on the media's timeline. If no media is present on the element, or
        // the media is not valid, the returned value is NaN. If the media has no known end (such as
        // for live streams of unknown duration, web radio, media incoming from WebRTC, and so
        // forth), this value is +Infinity.
        // duration Read only

        /// If specified, the audio player will automatically seek back to the start upon reaching
        /// the end of the audio.
        loop_(bool)

        /// Indicates whether the audio will be initially silenced. Its default value is false.
        muted(bool)

        /// This enumerated attribute is intended to provide a hint to the browser about what the
        /// author thinks will lead to the best user experience. It may have one of the following
        /// values:
        ///
        /// * `none`: Indicates that the audio should not be preloaded.
        /// * `metadata`: Indicates that only audio metadata (e.g. length) is fetched.
        /// * `auto`: Indicates that the whole audio file can be downloaded, even if the user is not
        ///   expected to use it.
        /// * empty string: A synonym of the auto value.
        ///
        /// The default value is different for each browser. The spec advises it to be set to
        /// metadata.
        ///
        /// Usage notes:
        ///
        /// The autoplay attribute has precedence over preload. If autoplay is specified, the
        /// browser would obviously need to start downloading the audio for playback.
        ///
        /// The browser is not forced by the specification to follow the value of this attribute; it
        /// is a mere hint.
        preload

        /// The URL of the audio to embed. This is subject to HTTP access controls. This is
        /// optional; you may instead use the <source> element within the audio block to specify the
        /// audio to embed.
        src
    }
}

html_element! {
    /// The [HTML `<img>` element][mdn] embeds an image into the document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img
    <img>

    categories {
        Flow, Phrasing, Embedded, Palpable,
        Interactive // if it has the `usemap` attribute
    }

    attributes {
        /// Defines an alternative text description of the image.
        ///
        /// > Note: Browsers do not always display images. For example:
        /// >
        /// > * Non-visual browsers (such as those used by people with visual impairments)
        /// > * The user chooses not to display images (saving bandwidth, privacy reasons)
        /// > * The image is invalid or an unsupported type
        /// > * In these cases, the browser may replace the image with the text in the element's alt
        /// attribute. For these reasons and others, provide a useful value for alt whenever
        /// possible.
        ///
        /// Omitting alt altogether indicates that the image is a key part of the content and no
        /// textual equivalent is available. Setting this attribute to an empty string (alt="")
        /// indicates that this image is not a key part of the content (it’s decoration or a
        /// tracking pixel), and that non-visual browsers may omit it from rendering. Visual
        /// browsers will also hide the broken image icon if the alt is empty and the image failed
        /// to display.
        ///
        /// This attribute is also used when copying and pasting the image to text, or saving a
        /// linked image to a bookmark.
        alt

        /// Indicates if the fetching of the image must be done using a CORS request. Image data
        /// from a CORS-enabled image returned from a CORS request can be reused in the <canvas>
        /// element without being marked "tainted".
        ///
        /// If the crossorigin attribute is not specified, then a non-CORS request is sent (without
        /// the Origin request header), and the browser marks the image as tainted and restricts
        /// access to its image data, preventing its usage in <canvas> elements.
        ///
        /// If the crossorigin attribute is specified, then a CORS request is sent (with the Origin
        /// request header); but if the server does not opt into allowing cross-origin access to the
        /// image data by the origin site (by not sending any Access-Control-Allow-Origin response
        /// header, or by not including the site's origin in any Access-Control-Allow-Origin
        /// response header it does send), then the browser marks the image as tainted and restricts
        /// access to its image data, preventing its usage in <canvas> elements.
        ///
        /// Allowed values:
        ///
        /// * `anonymous`: A CORS request is sent with credentials omitted (that is, no cookies,
        ///   X.509 certificates, or Authorization request header).
        /// * `use-credentials`: The CORS request is sent with any credentials included (that is,
        ///   cookies, X.509 certificates, and the `Authorization` request header). If the server
        ///   does not opt into sharing credentials with the origin site (by sending back the
        ///   `Access-Control-Allow-Credentials: true` response header), then the browser marks the
        ///   image as tainted and restricts access to its image data.
        ///
        /// If the attribute has an invalid value, browsers handle it as if the anonymous value was
        /// used.
        crossorigin

        /// Provides an image decoding hint to the browser. Allowed values:
        ///
        /// * `sync`: Decode the image synchronously, for atomic presentation with other content.
        /// * `async`: Decode the image asynchronously, to reduce delay in presenting other content.
        /// * `auto`: Default: no preference for the decoding mode. The browser decides what is best
        ///   for the user.
        decoding

        /// The intrinsic height of the image, in pixels. Must be an integer without a unit.
        height

        /// Indicates that the image is part of a server-side map. If so, the coordinates where the
        /// user clicked on the image are sent to the server.
        ///
        /// Note: This attribute is allowed only if the <img> element is a descendant of an <a>
        /// element with a valid href attribute. This gives users without pointing devices a
        /// fallback destination.
        ismap(bool)

        /// Indicates how the browser should load the image:
        ///
        /// * `eager`: Loads the image immediately, regardless of whether or not the image is
        ///   currently within the visible viewport (this is the default value).
        /// * `lazy`: Defers loading the image until it reaches a calculated distance from the
        ///   viewport, as defined by the browser. The intent is to avoid the network and storage
        ///   bandwidth needed to handle the image until it's reasonably certain that it will be
        ///   needed. This generally improves the performance of the content in most typical use
        ///   cases.
        ///
        /// > Note: Loading is only deferred when JavaScript is enabled. This is an anti-tracking
        /// measure, because if a user agent supported lazy loading when scripting is disabled, it
        /// would still be possible for a site to track a user's approximate scroll position
        /// throughout a session, by strategically placing images in a page's markup such that a
        /// server can track how many images are requested and when.
        loading

        /// One or more strings separated by commas, indicating a set of source sizes. Each source
        /// size consists of:
        ///
        /// * A media condition. This must be omitted for the last item in the list.
        /// * A source size value.
        ///
        /// Media Conditions describe properties of the viewport, not of the image. For example,
        /// (max-height: 500px) 1000px proposes to use a source of 1000px width, if the viewport is
        /// not higher than 500px.
        ///
        /// Source size values specify the intended display size of the image. User agents use the
        /// current source size to select one of the sources supplied by the srcset attribute, when
        /// those sources are described using width (w) descriptors. The selected source size
        /// affects the intrinsic size of the image (the image’s display size if no CSS styling is
        /// applied). If the srcset attribute is absent, or contains no values with a width
        /// descriptor, then the sizes attribute has no effect.
        sizes

        /// The image URL. Mandatory for the <img> element. On browsers supporting srcset, src is
        /// treated like a candidate image with a pixel density descriptor 1x, unless an image with
        /// this pixel density descriptor is already defined in srcset, or unless srcset contains w
        /// descriptors.
        src

        /// One or more strings separated by commas, indicating possible image sources for the user
        /// agent to use. Each string is composed of:
        ///
        /// * A URL to an image
        /// * Optionally, whitespace followed by one of:
        ///   * A width descriptor (a positive integer directly followed by w). The width descriptor
        ///     is divided by the source size given in the sizes attribute to calculate the
        ///     effective pixel density.
        ///   * A pixel density descriptor (a positive floating point number directly followed by
        ///     x).
        ///   * If no descriptor is specified, the source is assigned the default descriptor of 1x.
        ///
        /// It is incorrect to mix width descriptors and pixel density descriptors in the same
        /// srcset attribute. Duplicate descriptors (for instance, two sources in the same srcset
        /// which are both described with 2x) are also invalid.
        ///
        /// The user agent selects any of the available sources at its discretion. This provides
        /// them with significant leeway to tailor their selection based on things like user
        /// preferences or bandwidth conditions. See our Responsive images tutorial for an example.
        srcset

        /// The intrinsic width of the image in pixels. Must be an integer without a unit.
        width

        /// The partial URL (starting with #) of an image map associated with the element.
        ///
        /// Note: You cannot use this attribute if the <img> element is inside an <a> or <button>
        /// element.
        usemap
    }
}

html_element! {
    /// The [HTML `<map>` element][mdn] is used with [`<area>`][area] elements to define an image
    /// map (a clickable link area).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map
    /// [area]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/area
    <map>

    categories {
        Flow, Phrasing, Palpable
    }

    attributes {
        /// The name attribute gives the map a name so that it can be referenced. The attribute must
        /// be present and must have a non-empty value with no space characters. The value of the
        /// name attribute must not be a compatibility-caseless match for the value of the name
        /// attribute of another <map> element in the same document. If the id attribute is also
        /// specified, both attributes must have the same value.
        name
    }
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
    <track>

    attributes {
        /// This attribute indicates that the track should be enabled unless the user's preferences
        /// indicate that another track is more appropriate. This may only be used on one track
        /// element per media element.
        default(bool)

        /// How the text track is meant to be used. If omitted the default kind is subtitles. If the
        /// attribute is not present, it will use the subtitles. If the attribute contains an
        /// invalid value, it will use metadata. The following keywords are allowed:

        /// Subtitles provide translation of content that cannot be understood by the viewer. For
        /// example dialogue or text that is not English in an English language film.
        ///
        /// Subtitles may contain additional content, usually extra background information. For
        /// example the text at the beginning of the Star Wars films, or the date, time, and
        /// location of a scene.
        subtitles

        /// Closed captions provide a transcription and possibly a translation of audio.
        ///
        /// It may include important non-verbal information such as music cues or sound effects. It
        /// may indicate the cue's source (e.g. music, text, character).
        ///
        /// Suitable for users who are deaf or when the sound is muted.
        captions

        /// Textual description of the video content.
        ///
        /// * `descriptions`: Suitable for users who are blind or where the video cannot be seen.
        /// * `chapters`: Chapter titles are intended to be used when the user is navigating the
        ///   media resource.
        /// * `metadata`: Tracks used by scripts. Not visible to the user.
        /// * `label`: A user-readable title of the text track which is used by the browser when
        ///   listing available text tracks.
        kind

        /// Address of the track (.vtt file). Must be a valid URL. This attribute must be specified
        /// and its URL value must have the same origin as the document — unless the <audio> or
        /// <video> parent element of the track element has a crossorigin attribute.
        src

        /// Language of the track text data. It must be a valid BCP 47 language tag. If the kind
        /// attribute is set to subtitles, then srclang must be defined.
        srclang
    }
}

html_element! {
    /// The [HTML Video element (`<video>`)][mdn] embeds a media player which supports video
    /// playback into the document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video
    <video>

    categories {
        Flow, Phrasing, Embedded,
        // if it has a `controls` attribute:
        Interactive, Palpable
    }

    children {
        tags {
            <track>,
            <source> // if the element doesn't have a src attribute
        }
    }

    attributes {
        /// If specified, the video automatically begins to play back as soon as it can do so
        /// without stopping to finish loading the data.
        ///
        /// Note: Sites that automatically play audio (or videos with an audio track) can be an
        /// unpleasant experience for users, so should be avoided when possible. If you must offer
        /// autoplay functionality, you should make it opt-in (requiring a user to specifically
        /// enable it). However, this can be useful when creating media elements whose source will
        /// be set at a later time, under user control. See our autoplay guide for additional
        /// information about how to properly use autoplay.
        ///
        /// To disable video autoplay, autoplay="false" will not work; the video will autoplay if
        /// the attribute is there in the <video> tag at all. To remove autoplay, the attribute
        /// needs to be removed altogether.
        autoplay(bool)

        /// An attribute you can read to determine the time ranges of the buffered media. This
        /// attribute contains a TimeRanges object.
        buffered

        /// If this attribute is present, the browser will offer controls to allow the user to
        /// control video playback, including volume, seeking, and pause/resume playback.
        controls(bool)

        /// This enumerated attribute indicates whether to use CORS to fetch the related image.
        /// CORS-enabled resources can be reused in the <canvas> element without being tainted. The
        /// allowed values are:
        ///
        /// * `anonymous`: Sends a cross-origin request without a credential. In other words, it
        ///   sends the `Origin: HTTP` header without a cookie, X.509 certificate, or performing
        ///   HTTP Basic authentication. If the server does not give credentials to the origin site
        ///   (by not setting the `Access-Control-Allow-Origin: HTTP` header), the image will be
        ///   tainted, and its usage restricted.
        /// * `use-credentials`: Sends a cross-origin request with a credential. In other words, it
        ///   sends the Origin: HTTP header with a cookie, a certificate, or performing HTTP Basic
        ///   authentication. If the server does not give credentials to the origin site (through
        ///   `Access-Control-Allow-Credentials: HTTP` header), the image will be tainted and its
        ///   usage restricted.
        ///
        /// When not present, the resource is fetched without a CORS request (i.e. without sending
        /// the `Origin: HTTP` header), preventing its non-tainted used in <canvas> elements. If
        /// invalid, it is handled as if the enumerated keyword anonymous was used.
        crossorigin

        /// Reading currentTime returns a double-precision floating-point value indicating the
        /// current playback position of the media specified in seconds. If the media has not
        /// started playing yet, the time offset at which it will begin is returned. Setting
        /// currentTime sets the current playback position to the given time and seeks the media to
        /// that position if the media is currently loaded.
        ///
        /// If the media is being streamed, it's possible that the user agent may not be able to
        /// obtain some parts of the resource if that data has expired from the media buffer. Other
        /// media may have a media timeline that doesn't start at 0 seconds, so setting currentTime
        /// to a time before that would fail. The getStartDate() method can be used to determine the
        /// beginning point of the media timeline's reference frame.
        current_time

        // A double-precision floating-point value which indicates the duration (total length) of
        // the media in seconds, on the media's timeline. If no media is present on the element, or
        // the media is not valid, the returned value is NaN. If the media has no known end (such as
        // for live streams of unknown duration, web radio, media incoming from WebRTC, and so
        // forth), this value is +Infinity.
        // duration

        /// The height of the video's display area, in CSS pixels (absolute values only; no
        /// percentages.)
        height

        /// If specified, the browser will automatically seek back to the start upon reaching the
        /// end of the video.
        loop_(bool)

        /// Indicates the default setting of the audio contained in the video. If set, the audio
        /// will be initially silenced. Its default value is false, meaning that the audio will be
        /// played when the video is played.
        muted(bool)

        /// Indicating that the video is to be played "inline", that is within the element's
        /// playback area. Note that the absence of this attribute does not imply that the video
        /// will always be played in fullscreen.
        playsinline(bool)

        /// A URL for an image to be shown while the video is downloading. If this attribute isn't
        /// specified, nothing is displayed until the first frame is available, then the first frame
        /// is shown as the poster frame.
        poster

        /// This enumerated attribute is intended to provide a hint to the browser about what the
        /// author thinks will lead to the best user experience with regards to what content is
        /// loaded before the video is played. It may have one of the following values:
        ///
        /// * `none`: Indicates that the video should not be preloaded.
        /// * `metadata`: Indicates that only video metadata (e.g. length) is fetched.
        /// * `auto`: Indicates that the whole video file can be downloaded, even if the user is not
        ///   expected to use it.
        /// * empty string: Synonym of the auto value.
        ///
        /// The default value is different for each browser. The spec advises it to be set to
        /// metadata.
        ///
        /// > Note:
        /// >
        /// > The autoplay attribute has precedence over preload. If autoplay is specified, the
        /// browser would obviously need to start downloading the video for playback.
        /// >
        /// > The specification does not force the browser to follow the value of this attribute; it
        /// is a mere hint.
        preload

        /// The URL of the video to embed. This is optional; you may instead use the <source>
        /// element within the video block to specify the video to embed.
        src

        /// The width of the video's display area, in CSS pixels (absolute values only; no
        /// percentages).
        width
    }
}
