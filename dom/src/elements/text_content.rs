//! Use HTML text content elements to organize blocks or sections of content
//! placed between the opening <body> and closing </body> tags. Important for
//! accessibility and SEO, these elements identify the purpose or structure of
//! that content.

html_element! {
    /// The [HTML `<blockquote>` element][mdn] (or *HTML Block Quotation Element*) indicates that
    /// the enclosed text is an extended quotation. Usually, this is rendered visually by
    /// indentation. A URL for the source of the quotation may be given using the `cite` attribute,
    /// while a text representation of the source can be given using the [`<cite>`][cite] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote
    /// [cite]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/cite
    <blockquote>

    attributes {
        /// A URL that designates a source document or message for the information quoted. This
        /// attribute is intended to point to information explaining the context or the reference
        /// for the quote.
        cite
    }
}

html_element! {
    /// The [HTML `<dd>` element][mdn] provides the description, definition, or value for the
    /// preceding term ([`<dt>`][dt]) in a description list ([`<dl>`][dl]).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dd
    /// [dt]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dt
    /// [dl]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl
    <dd>
}

html_element! {
    /// The [HTML Content Division element (`<div>`)][mdn] is the generic container for flow
    /// content. It has no effect on the content or layout until styled using [CSS].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div
    /// [CSS]: https://developer.mozilla.org/en-US/docs/Glossary/CSS
    <div>
}

html_element! {
    /// The [HTML `<dl>` element][mdn] represents a description list. The element encloses a list of
    /// groups of terms (specified using the [`<dt>`][dt] element) and descriptions (provided by
    /// [`<dd>`][dd] elements). Common uses for this element are to implement a glossary or to
    /// display metadata (a list of key-value pairs).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl
    /// [dt]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dt
    /// [dd]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dd
    <dl>
}

html_element! {
    /// The [HTML `<dt>` element][mdn] specifies a term in a description or definition list, and as
    /// such must be used inside a [`<dl>`][dl] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dt
    /// [dl]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl
    <dt>
}

html_element! {
    /// The [HTML `<figcaption>` or Figure Caption element][mdn] represents a caption or legend
    /// describing the rest of the contents of its parent [`<figure>`][figure] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figcaption
    /// [figure]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figure
    <figcaption>
}

html_element! {
    /// The [HTML `<figure>` (Figure With Optional Caption) element][mdn] represents self-contained
    /// content, potentially with an optional caption, which is specified using the
    /// ([`<figcaption>`][figcaption]) element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figure
    /// [figcaption]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figcaption
    <figure>
}

html_element! {
    /// The [HTML `<hr>` element][mdn] represents a thematic break between paragraph-level elements:
    /// for example, a change of scene in a story, or a shift of topic within a section.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr
    <hr>
}

html_element! {
    /// The [HTML `<li>` element][mdn] is used to represent an item in a list.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li
    <li>
}

html_element! {
    /// The [HTML `<ol>` element][mdn] represents an ordered list of items, typically rendered as a
    /// numbered list.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol
    <ol>

    attributes {
        /// Specifies that the list’s items are in reverse order. Items will be numbered from high
        /// to low.
        reversed(bool)

        /// An integer to start counting from for the list items. Always an Arabic numeral (1, 2, 3,
        /// etc.), even when the numbering type is letters or Roman numerals. For example, to start
        /// numbering elements from the letter "d" or the Roman numeral "iv," use start="4".
        start(u32)

        /// Sets the numbering type:
        ///
        /// * `a` for lowercase letters
        /// * `A` for uppercase letters
        /// * `i` for lowercase Roman numerals
        /// * `I` for uppercase Roman numerals
        /// * `1` for numbers (default)
        ///
        /// The specified type is used for the entire list unless a different type attribute is used
        /// on an enclosed <li> element.
        ///
        /// > Note: Unless the type of the list number matters (like legal or technical documents
        /// where items are referenced by their number/letter), use the CSS list-style-type property
        /// instead.
        type_
    }
}

html_element! {
    /// The [HTML `<p>` element][mdn] represents a paragraph.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p
    <p>
}

html_element! {
    /// The [HTML `<pre>` element][mdn] represents preformatted text which is to be presented
    /// exactly as written in the HTML file.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre
    <pre>
}

html_element! {
    /// The [HTML `<ul>` element][mdn] represents an unordered list of items, typically rendered as
    /// a bulleted list.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul
    <ul>
}
