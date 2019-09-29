//! Element definitions generated from the listing on [MDN]. Because topologically-aware functions
//! are called-by-macro today, the element macros generated here appear in the root module.
//!
//! [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element

macro_rules! element_macro {
    (
        $(#[$outer:meta])*
        $name:ident
    ) => {
        $(#[$outer])*
        #[macro_export]
        macro_rules! $name {
            ($with_elem:expr) => {
                $crate::element!(stringify!($name), $with_elem)
            };
        }
    };
}

element_macro! {
    /// The [`<html>` element][mdn] represents the root (top-level element) of an HTML document,
    /// so it is also referred to as the *root element*. All other elements must be descendants of
    /// this element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/html
    html
}

element_macro! {
    /// The [HTML `<base> element`][mdn] specifies the base URL to use for all relative URLs
    /// contained within a document. There can be only one `<base>` element in a document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base
    base
}

element_macro! {
    /// The [HTML `<head>` element][mdn] contains machine-readable information ([metadata]) about
    /// the document, like its [title], [scripts], and [style sheets].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/head
    /// [metadata]: https://developer.mozilla.org/en-US/docs/Glossary/metadata
    /// [title]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title
    /// [scripts]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script
    /// [style sheets]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style
    head
}

element_macro! {
    /// The [HTML External Resource Link element (`<link>`)][mdn] specifies relationships between
    /// the current document and an external resource. This element is most commonly used to link to
    /// [stylesheets], but is also used to establish site icons (both "favicon" style icons and
    /// icons for the home screen and apps on mobile devices) among other things.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link
    /// [stylesheets]: https://developer.mozilla.org/en-US/docs/Glossary/CSS
    link
}

element_macro! {
    /// The [HTML `<meta>` element][mdn] represents [metadata] that cannot be represented by other
    /// HTML meta-related elements, like [`<base>`], [`<link>`], [`<script>`], [`<style>`] or
    /// [`<title>`].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta
    /// [metadata]: https://developer.mozilla.org/en-US/docs/Glossary/Metadata
    /// [base]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base
    /// [link]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link
    /// [script]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script
    /// [style]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style
    /// [title]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title
    meta
}

element_macro! {
    /// The [HTML `<style>` element][mdn] contains style information for a document, or part of a
    /// document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style
    style
}

element_macro! {
    /// The [HTML Title element (`<title>`)][mdn] defines the document's title that is shown in a
    /// [browser]'s title bar or a page's tab.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title
    /// [browser]: https://developer.mozilla.org/en-US/docs/Glossary/Browser
    title
}

element_macro! {
    /// The [HTML `<body>` element][mdn] represents the content of an HTML document. There can be
    /// only one `<body>` element in a document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/body
    body
}

element_macro! {
    /// The [HTML `<address>` element][mdn] indicates that the enclosed HTML provides contact
    /// information for a person or people, or for an organization.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/address
    address
}

element_macro! {
    /// The [HTML `<article>` element][mdn] represents a self-contained composition in a document,
    /// page, application, or site, which is intended to be independently distributable or reusable
    /// (e.g., in syndication).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/article
    article
}

element_macro! {
    /// The [HTML `<aside>` element][mdn] represents a portion of a document whose content is only
    /// indirectly related to the document's main content.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside
    aside
}

element_macro! {
    /// The [HTML `<footer>` element][mdn] represents a footer for its nearest [sectioning content]
    /// or [sectioning root] element. A footer typically contains information about the author of
    /// the section, copyright data or links to related documents.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/footer
    /// [sectioning content]: https://developer.mozilla.org/en-US/docs/Web/Guide/HTML/Content_categories#Sectioning_content
    /// [sectioning root]: https://developer.mozilla.org/en-US/docs/Web/Guide/HTML/Sections_and_Outlines_of_an_HTML5_document#Sectioning_roots
    footer
}

element_macro! {
    /// The [HTML `<header>` element][mdn] represents introductory content, typically a group of
    /// introductory or navigational aids. It may contain some heading elements but also a logo, a
    /// search form, an author name, and other elements.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/header
    header
}

element_macro! {
    /// The [HTML `<h1>`–`<h6>` elements][mdn] represent six levels of section headings. `<h1>` is
    /// the highest section level and `<h6>` is the lowest.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h1
    h1
}

element_macro! {
    /// The [HTML `<h1>`–`<h6>` elements][mdn] represent six levels of section headings. `<h1>` is
    /// the highest section level and `<h6>` is the lowest.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h2
    h2
}

element_macro! {
    /// The [HTML `<h1>`–`<h6>` elements][mdn] represent six levels of section headings. `<h1>` is
    /// the highest section level and `<h6>` is the lowest.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h3
    h3
}

element_macro! {
    /// The [HTML `<h1>`–`<h6>` elements][mdn] represent six levels of section headings. `<h1>` is
    /// the highest section level and `<h6>` is the lowest.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h4
    h4
}

element_macro! {
    /// The [HTML `<h1>`–`<h6>` elements][mdn] represent six levels of section headings. `<h1>` is
    /// the highest section level and `<h6>` is the lowest.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h5
    h5
}

element_macro! {
    /// The [HTML `<h1>`–`<h6>` elements][mdn] represent six levels of section headings. `<h1>` is
    /// the highest section level and `<h6>` is the lowest.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/h6
    h6
}

element_macro! {
    /// The [HTML `<hgroup>` element][mdn] represents a multi-level heading for a section of a
    /// document. It groups a set of [`<h1>–<h6>`][heading] elements.
    ///
    /// mdn: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hgroup
    /// heading: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements
    hgroup
}

element_macro! {
    /// The [HTML `<nav>` element][mdn] represents a section of a page whose purpose is to provide
    /// navigation links, either within the current document or to other documents. Common examples
    /// of navigation sections are menus, tables of contents, and indexes.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/nav
    nav
}

element_macro! {
    /// The [HTML `<section>` element][mdn] represents a standalone section — which doesn't have a
    /// more specific semantic element to represent it — contained within an HTML document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/section
    section
}

element_macro! {
    /// The [HTML `<blockquote>` element][mdn] (or *HTML Block Quotation Element*) indicates that
    /// the enclosed text is an extended quotation. Usually, this is rendered visually by
    /// indentation. A URL for the source of the quotation may be given using the `cite` attribute,
    /// while a text representation of the source can be given using the [`<cite>`][cite] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote
    /// [cite]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/cite
    blockquote
}

element_macro! {
    /// The [HTML `<dd>` element][mdn] provides the description, definition, or value for the
    /// preceding term ([`<dt>`][dt]) in a description list ([`<dl>`][dl]).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dd
    /// [dt]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dt
    /// [dl]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl
    dd
}

element_macro! {
    /// The [HTML Content Division element (`<div>`)][mdn] is the generic container for flow
    /// content. It has no effect on the content or layout until styled using [CSS].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div
    /// [CSS]: https://developer.mozilla.org/en-US/docs/Glossary/CSS
    div
}

element_macro! {
    /// The [HTML `<dl>` element][mdn] represents a description list. The element encloses a list of
    /// groups of terms (specified using the [`<dt>`][dt] element) and descriptions (provided by
    /// [`<dd>`][dd] elements). Common uses for this element are to implement a glossary or to
    /// display metadata (a list of key-value pairs).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl
    /// [dt]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dt
    /// [dd]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dd
    dl
}

element_macro! {
    /// The [HTML `<dt>` element][mdn] specifies a term in a description or definition list, and as
    /// such must be used inside a [`<dl>`][dl] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dt
    /// [dl]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl
    dt
}

element_macro! {
    /// The [HTML `<figcaption>` or Figure Caption element][mdn] represents a caption or legend
    /// describing the rest of the contents of its parent [`<figure>`][figure] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figcaption
    /// [figure]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figure
    figcaption
}

element_macro! {
    /// The [HTML `<figure>` (Figure With Optional Caption) element][mdn] represents self-contained
    /// content, potentially with an optional caption, which is specified using the
    /// ([`<figcaption>`][figcaption]) element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figure
    /// [figcaption]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figcaption
    figure
}

element_macro! {
    /// The [HTML `<hr>` element][mdn] represents a thematic break between paragraph-level elements:
    /// for example, a change of scene in a story, or a shift of topic within a section.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr
    hr
}

element_macro! {
    /// The [HTML `<li>` element][mdn] is used to represent an item in a list.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li
    li
}

element_macro! {
    /// The [HTML `<main>` element][mdn] represents the dominant content of the [`<body>`][body] of
    /// a document. The main content area consists of content that is directly related to or expands
    /// upon the central topic of a document, or the central functionality of an application.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/main
    /// [body]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/body
    main
}

element_macro! {
    /// The [HTML `<ol>` element][mdn] represents an ordered list of items, typically rendered as a
    /// numbered list.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol
    ol
}

element_macro! {
    /// The [HTML `<p>` element][mdn] represents a paragraph.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p
    p
}

element_macro! {
    /// The [HTML `<pre>` element][mdn] represents preformatted text which is to be presented
    /// exactly as written in the HTML file.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre
    pre
}

element_macro! {
    /// The [HTML `<ul>` element][mdn] represents an unordered list of items, typically rendered as
    /// a bulleted list.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul
    ul
}

element_macro! {
    /// The [HTML `<a>` element (or *anchor* element)][mdn], along with its href attribute, creates
    /// a hyperlink to other web pages, files, locations within the same page, email addresses, or
    /// any other URL.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a
    a
}

element_macro! {
    /// The [HTML Abbreviation element (`<abbr>`)][mdn] represents an abbreviation or acronym; the
    /// optional [`title`][title] attribute can provide an expansion or description for the
    /// abbreviation.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/abbr
    /// [title]: https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes#attr-title
    abbr
}

element_macro! {
    /// The [HTML Bring Attention To element (`<b>`)][mdn] is used to draw the reader's attention to
    /// the element's contents, which are not otherwise granted special importance.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/b
    b
}

element_macro! {
    /// The [HTML Bidirectional Isolate element (`<bdi>`)][mdn] tells the browser's bidirectional
    /// algorithm to treat the text it contains in isolation from its surrounding text.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/bdi
    bdi
}

element_macro! {
    /// The [HTML Bidirectional Text Override element (`<bdo>`)][mdn] overrides the current
    /// directionality of text, so that the text within is rendered in a different direction.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/bdo
    bdo
}

element_macro! {
    /// The [HTML `<br>` element][mdn] produces a line break in text (carriage-return). It is useful
    /// for writing a poem or an address, where the division of lines is significant.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/br
    br
}

element_macro! {
    /// The [HTML Citation element (`<cite>`)][mdn] is used to describe a reference to a cited
    /// creative work, and must include the title of that work.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/cite
    cite
}

element_macro! {
    /// The [HTML `<code>` element][mdn] displays its contents styled in a fashion intended to
    /// indicate that the text is a short fragment of computer code.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/code
    code
}

element_macro! {
    /// The [HTML `<data>` element][mdn] links a given content with a machine-readable translation.
    /// If the content is time- or date-related, the [`<time>`][time] element must be used.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/data
    /// [time]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time
    data
}

element_macro! {
    /// The [HTML Definition element (`<dfn>`)][mdn] is used to indicate the term being defined
    /// within the context of a definition phrase or sentence.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dfn
    dfn
}

element_macro! {
    /// The [HTML `<em>` element][mdn] marks text that has stress emphasis. The `<em>` element can
    /// be nested, with each level of nesting indicating a greater degree of emphasis.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em
    em
}

element_macro! {
    /// The [HTML `<i>` element][mdn] represents a range of text that is set off from the normal
    /// text for some reason. Some examples include technical terms, foreign language phrases, or
    /// fictional character thoughts. It is typically displayed in italic type.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/i
    i
}

element_macro! {
    /// The [HTML Keyboard Input element (`<kbd>`)][mdn] represents a span of inline text denoting
    /// textual user input from a keyboard, voice input, or any other text entry device.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/kbd
    kbd
}

element_macro! {
    /// The [HTML Mark Text element (`<mark>`)][mdn] represents text which is marked or highlighted
    /// for reference or notation purposes, due to the marked passage's relevance or importance in
    /// the enclosing context.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/mark
    mark
}

element_macro! {
    /// The [HTML `<q>` element][mdn]  indicates that the enclosed text is a short inline quotation.
    /// Most modern browsers implement this by surrounding the text in quotation marks.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q
    q
}

element_macro! {
    /// The [HTML Ruby Base (`<rb>`) element][mdn] is used to delimit the base text component of
    /// a [`<ruby>`][ruby] annotation, i.e. the text that is being annotated.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rb
    /// [ruby]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby
    rb
}

element_macro! {
    /// The [HTML Ruby Fallback Parenthesis (`<rp>`) element][mdn] is used to provide fall-back
    /// parentheses for browsers that do not support display of ruby annotations using the
    /// [`<ruby>`][ruby] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rp
    /// [ruby]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby
    rp
}

element_macro! {
    /// The [HTML Ruby Text (`<rt>`) element][mdn] specifies the ruby text component of a ruby
    /// annotation, which is used to provide pronunciation, translation, or transliteration
    /// information for East Asian typography. The `<rt>` element must always be contained within a
    /// [`<ruby>`][ruby] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rt
    /// [ruby]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby
    rt
}

element_macro! {
    /// The [HTML Ruby Text Container (`<rtc>`) element][mdn] embraces semantic annotations of
    /// characters presented in a ruby of [`<rb>`][rb] elements used inside of [`<ruby>`][ruby]
    /// element. [`<rb>`][rb] elements can have both pronunciation ([`<rt>`][rt]) and semantic
    /// ([`<rtc>`][rtc]) annotations.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rtc
    /// [rb]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rb
    /// [ruby]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby
    /// [rt]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rt
    /// [rtc]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rtc
    rtc
}

element_macro! {
    /// The [HTML `<ruby>` element][mdn] represents a ruby annotation. Ruby annotations are for
    /// showing pronunciation of East Asian characters.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby
    ruby
}

element_macro! {
    /// The [HTML `<s>` element][mdn] renders text with a strikethrough, or a line through it. Use
    /// the `<s>` element to represent things that are no longer relevant or no longer accurate.
    /// However, `<s>` is not appropriate when indicating document edits; for that, use the
    /// [`<del>`][del] and [`<ins>`][ins] elements, as appropriate.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/s
    /// [del]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del
    /// [ins]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ins
    s
}

element_macro! {
    /// The [HTML Sample Element (`<samp>`)][mdn] is used to enclose inline text which represents
    /// sample (or quoted) output from a computer program.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/samp
    samp
}

element_macro! {
    /// The [HTML `<small>` element][mdn] represents side-comments and small print, like copyright
    /// and legal text, independent of its styled presentation. By default, it renders text within
    /// it one font-size small, such as from `small` to `x-small`.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/small
    small
}

element_macro! {
    /// The [HTML `<span>` element][mdn] is a generic inline container for phrasing content, which
    /// does not inherently represent anything. It can be used to group elements for styling
    /// purposes (using the [`class`][class] or [`id`][id] attributes), or because they share
    /// attribute values, such as [`lang`][lang].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span
    /// [class]: https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes#attr-class
    /// [id]: https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes#attr-id
    /// [lang]: https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes#attr-lang
    span
}

element_macro! {
    /// The [HTML Strong Importance Element (`<strong>`)][mdn] indicates that its contents have
    /// strong importance, seriousness, or urgency. Browsers typically render the contents in bold
    /// type.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong
    strong
}

element_macro! {
    /// The [HTML Subscript element (`<sub>`)][mdn] specifies inline text which should be displayed
    /// as subscript for solely typographical reasons.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sub
    sub
}

element_macro! {
    /// The [HTML Superscript element (`<sup>`)][mdn] specifies inline text which is to be displayed
    /// as superscript for solely typographical reasons.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sup
    sup
}

element_macro! {
    /// The [HTML `<time>` element][mdn] represents a specific period in time.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time
    time
}

element_macro! {
    /// The [HTML Unarticulated Annotation Element (`<u>`)][mdn] represents a span of inline text
    /// which should be rendered in a way that indicates that it has a non-textual annotation.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/u
    u
}

element_macro! {
    /// The [HTML Variable element (`<var>`)][mdn] represents the name of a variable in a
    /// mathematical expression or a programming context.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/var
    var
}

element_macro! {
    /// The [HTML `<wbr>` element][mdn] represents a word break opportunity—a position within text
    /// where the browser may optionally break a line, though its line-breaking rules would not
    /// otherwise create a break at that location.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/wbr
    wbr
}

element_macro! {
    /// The [HTML `<area>` element][mdn] defines a hot-spot region on an image, and optionally
    /// associates it with a [hypertext link]. This element is used only within a [`<map>`][map]
    /// element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/area
    /// [hypertext link]: https://developer.mozilla.org/en-US/docs/Glossary/Hyperlink
    /// [map]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map
    area
}

element_macro! {
    /// The [HTML `<audio>` element][mdn] is used to embed sound content in documents. It may
    /// contain one or more audio sources, represented using the `src` attribute or the
    /// [`<source>`][source] element: the browser will choose the most suitable one. It can also be
    /// the destination for streamed media, using a [`MediaStream`][stream].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio
    /// [source]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source
    /// [stream]: https://developer.mozilla.org/en-US/docs/Web/API/MediaStream
    audio
}

element_macro! {
    /// The [HTML `<img>` element][mdn] embeds an image into the document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img
    img
}

element_macro! {
    /// The [HTML `<map>` element][mdn] is used with [`<area>`][area] elements to define an image
    /// map (a clickable link area).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/map
    /// [area]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/area
    map
}

element_macro! {
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
    track
}

element_macro! {
    /// The [HTML Video element (`<video>`)][mdn] embeds a media player which supports video
    /// playback into the document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video
    video
}

element_macro! {
    /// The [HTML `<embed>` element][mdn] embeds external content at the specified point in the
    /// document. This content is provided by an external application or other source of interactive
    /// content such as a browser plug-in.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/embed
    embed
}

element_macro! {
    /// The [HTML Inline Frame element (`<iframe>`)][mdn] represents a nested [browsing context],
    /// embedding another HTML page into the current one.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe
    /// [browsing context]: https://developer.mozilla.org/en-US/docs/Glossary/browsing_context
    iframe
}

element_macro! {
    /// The [HTML `<object>` element][mdn] represents an external resource, which can be treated as
    /// an image, a nested browsing context, or a resource to be handled by a plugin.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/object
    object
}

element_macro! {
    /// The [HTML `<param>` element][param] defines parameters for an [`<object>`][object] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/param
    /// [object]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/object
    param
}

element_macro! {
    /// The [HTML `<picture>` element][mdn] contains zero or more [`<source>`][source] elements and
    /// one [`<img>`][img] element to provide versions of an image for different display/device
    /// scenarios.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture
    /// [source]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source
    /// [img]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img
    picture
}

element_macro! {
    /// The [HTML `<source>` element][source] specifies multiple media resources for the
    /// [`<picture>`][picture], the [`<audio>`][audio] element, or the [`<video>`][video] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source
    /// [picture]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture
    /// [audio]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio
    /// [video]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video
    source
}

element_macro! {
    /// Use the [HTML `<canvas>` element][mdn] with either the [canvas scripting API][api] or the
    /// [WebGL API][gl] to draw graphics and animations.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas
    /// [api]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
    /// [gl]: https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API
    canvas
}

element_macro! {
    /// The [HTML `<noscript>` element][mdn] defines a section of HTML to be inserted if a script
    /// type on the page is unsupported or if scripting is currently turned off in the browser.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/noscript
    noscript
}

element_macro! {
    /// The [HTML `<script>` element][mdn] is used to embed or reference executable code; this is
    /// typically used to embed or refer to JavaScript code.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script
    script
}

element_macro! {
    /// The [HTML `<del>` element][mdn] represents a range of text that has been deleted from a
    /// document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del
    del
}

element_macro! {
    /// The [HTML `<ins>` element][mdn] represents a range of text that has been added to a
    /// document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ins
    ins
}

element_macro! {
    /// The [HTML Table Caption element (`<caption>`)][mdn] specifies the caption (or title) of a
    /// table, and if used is *always* the first child of a [`<table>`][table].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/caption
    /// [table]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table
    caption
}

element_macro! {
    /// The [HTML `<col>` element][mdn] defines a column within a table and is used for defining
    /// common semantics on all common cells. It is generally found within a [`<colgroup>`][cg]
    /// element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/col
    /// [cg]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/colgroup
    col
}

element_macro! {
    /// The [HTML `<colgroup>` element][mdn] defines a group of columns within a table.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/colgroup
    colgroup
}

element_macro! {
    /// The [HTML `<table>` element][mdn] represents tabular data — that is, information presented
    /// in a two-dimensional table comprised of rows and columns of cells containing data.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table
    table
}

element_macro! {
    /// The [HTML Table Body element (`<tbody>`)][mdn] encapsulates a set of table rows
    /// ([`<tr>`][tr] elements), indicating that they comprise the body of the table
    /// ([`<table>`][table]).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tbody
    /// [tr]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr
    /// [table]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table
    tbody
}

element_macro! {
    /// The [HTML `<td>` element][mdn] defines a cell of a table that contains data. It participates
    /// in the *table model*.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td
    td
}

element_macro! {
    /// The [HTML `<tfoot>` element][mdn] defines a set of rows summarizing the columns of the
    /// table.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tfoot
    tfoot
}

element_macro! {
    /// The [HTML `<th>` element][mdn] defines a cell as header of a group of table cells. The exact
    /// nature of this group is defined by the [`scope`][scope] and [`headers`][headers] attributes.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th
    /// [scope]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th#attr-scope
    /// [headers]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th#attr-headers
    th
}

element_macro! {
    /// The [HTML `<thead>` element][mdn] defines a set of rows defining the head of the columns of
    /// the table.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/thead
    thead
}

element_macro! {
    /// The [HTML `<tr>` element][mdn] defines a row of cells in a table. The row's cells can then
    /// be established using a mix of [`<td>`][td] (data cell) and [`<th>`][th] (header cell)
    /// elements.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr
    /// [td]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td
    /// [th]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th
    tr
}

element_macro! {
    /// The HTML `<button>` element represents a clickable button, which can be used in <a href="https://developer.mozilla.org/en-US/docs/Learn/HTML/Forms">forms</a> or anywhere in a document that needs simple, standard button functionality.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button
    button
}

element_macro! {
    /// The HTML `<datalist>` element contains a set of <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/option" title="The HTML <option> element is used to define an item contained in a <select>, an <optgroup>, or a <datalist>&nbsp;element. As such,&nbsp;<option>&nbsp;can represent menu items in popups and other lists of items in an HTML document.">`<option>`</a> elements that represent the values available for other controls.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/datalist
    datalist
}

element_macro! {
    /// The HTML `<fieldset>` element is used to group several controls as well as labels (<a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label" title="The HTML <label> element represents a caption for an item in a user interface.">`<label>`</a>) within a web form.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/fieldset
    fieldset
}

element_macro! {
    /// The HTML `<form>` element represents a document section that contains interactive controls for submitting information to a web server.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form
    form
}

element_macro! {
    /// The HTML `<input>` element is used to create interactive controls for web-based forms in order to accept data from the user; a wide variety of types of input data and control widgets are available, depending on the device and <a class="glossaryLink" href="https://developer.mozilla.org/en-US/docs/Glossary/user_agent" title="user agent: A user agent is a computer program representing a person, for example, a browser in a Web context.">user agent</a>.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input
    input
}

element_macro! {
    /// The HTML `<label>` element represents a caption for an item in a user interface.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label
    label
}

element_macro! {
    /// The HTML `<legend>` element represents a caption for the content of its parent <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/fieldset" title="The HTML <fieldset> element is used to group several controls as well as labels (<label>) within a web form.">`<fieldset>`</a>.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/legend
    legend
}

element_macro! {
    /// The HTML `<meter>` element represents either a scalar value within a known range or a fractional value.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meter
    meter
}

element_macro! {
    /// The HTML `<optgroup>` element creates a grouping of options within a <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select" title="The HTML <select> element represents a control that provides a menu of options">`<select>`</a> element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/optgroup
    optgroup
}

element_macro! {
    /// The HTML `<option>` element is used to define an item contained in a <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select" title="The HTML <select> element represents a control that provides a menu of options">`<select>`</a>, an <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/optgroup" title="The HTML <optgroup> element creates a grouping of options within a <select> element.">`<optgroup>`</a>, or a <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/datalist" title="The HTML <datalist> element contains a set of <option> elements that represent the values available for other controls.">`<datalist>`</a>&nbsp;element. As such,&nbsp;`<option>`&nbsp;can represent menu items in popups and other lists of items in an HTML document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/option
    option
}

element_macro! {
    /// The HTML Output element (`<output>`) is a container element into which a site or app can inject the results of a calculation or the outcome of a user action.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/output
    output
}

element_macro! {
    /// The HTML `<progress>` element displays an indicator showing the completion progress of a task, typically displayed as a progress bar.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/progress
    progress
}

element_macro! {
    /// The HTML `<select>` element represents a control that provides a menu of options
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select
    select
}

element_macro! {
    /// The HTML `<textarea>` element represents a multi-line plain-text editing control, useful when you want to allow users to enter a sizeable amount of free-form text, for example a comment on a review or feedback form.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/textarea
    textarea
}

element_macro! {
    /// The HTML Details Element (`<details>`) creates a disclosure widget in which information is visible only when the widget is toggled into an "open" state.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details
    details
}

element_macro! {
    /// The HTML `<dialog>` element represents a dialog box or other interactive component, such as an inspector or window.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dialog
    dialog
}

element_macro! {
    /// The HTML `<menu>` element represents a group of commands that a user can perform or activate. This includes both list menus, which might appear across the top of a screen, as well as context menus, such as those that might appear underneath a button after it has been clicked.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/menu
    menu
}

element_macro! {
    /// The HTML `<menuitem>` element represents a command that a user is able to invoke through a popup menu. This includes context menus, as well as menus that might be attached to a menu button.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/menuitem
    menuitem
}

element_macro! {
    /// The HTML Disclosure Summary element (`<summary>`) element specifies a summary, caption, or legend for a <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details" title="The HTML Details Element (&amp;lt;details&amp;gt;) creates a disclosure widget in which information is visible only when the widget is toggled into an &quot;open&quot; state.">`<details>`</a> element's disclosure box.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/summary
    summary
}

element_macro! {
    /// The HTML `<slot>` element—part of the <a href="https://developer.mozilla.org/en-US/docs/Web/Web_Components">Web Components</a> technology suite—is a placeholder inside a web component that you can fill with your own markup, which lets you create separate DOM trees and present them together.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/slot
    slot
}

element_macro! {
    /// The HTML Content Template (`<template>`) element is a mechanism for holding <a class="glossaryLink" href="https://developer.mozilla.org/en-US/docs/Glossary/HTML" title="HTML: HTML (HyperText Markup Language) is a descriptive language that specifies webpage structure.">HTML</a> that is not to be rendered immediately when a page is loaded but may be instantiated subsequently during runtime using JavaScript.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template
    template
}

element_macro! {
    /// The HTML Acronym Element (`<acronym>`) allows authors to clearly indicate a sequence of characters that compose an acronym or abbreviation for a word. This element has been removed in HTML5. Use&nbsp;<a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/abbr" title="The HTML Abbreviation element (<abbr>) represents an abbreviation or acronym; the optional title attribute can provide an expansion or description for the abbreviation.">`<abbr>`</a> instead.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/acronym
    acronym
}

element_macro! {
    /// The HTML Command element (`<command>`) represents a command which the user can invoke. Commands are often used as part of a context menu or toolbar.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/command
    command
}

element_macro! {
    /// The *HTML Font Element* (`<font>`) defines the font size, color and face for its content.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/font
    font
}

element_macro! {
    /// `<frame>` is an HTML element which defines a particular area in which another HTML document can be displayed. A frame should be used within a <a href="https://developer.mozilla.org/en-US/docs/Web/HTML/Element/frameset" title="The HTML <frameset> element is used to contain <frame> elements.">`<frameset>`</a>.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/frame
    frame
}
