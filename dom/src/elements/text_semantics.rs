//! HTML inline text semantic define the meaning, structure, or style
//! of a word, line, or any arbitrary piece of text.
//!
//! Also includes elements that provide indications that specific parts of the
//! text have been altered.

html_element! {
    /// The [HTML `<a>` element (or *anchor* element)][mdn], along with its href attribute, creates
    /// a hyperlink to other web pages, files, locations within the same page, email addresses, or
    /// any other URL.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a
    <a>

    categories {
        Flow, Phrasing, Interactive, Palpable
    }

    children {
        categories {
            Flow
        }
    }

    attributes {
        /// Prompts the user to save the linked URL instead of navigating to it. Can be used with or
        /// without a value:
        ///
        /// * Without a value, the browser will suggest a filename/extension, generated from various
        ///   sources:
        ///   * The Content-Disposition HTTP header
        ///   * The final segment in the URL path
        ///   * The media type (from the (Content-Type header, the start of a data: URL, or
        ///     Blob.type for a blob: URL)
        /// * Defining a value suggests it as the filename. `/` and `\` characters are converted to
        ///   underscores (_). Filesystems may forbid other characters in filenames, so browsers
        ///   will adjust the suggested name if necessary.
        ///
        /// > Notes:
        /// > * download only works for same-origin URLs, or the blob: and data: schemes.
        /// > * If Content-Disposition has a different filename than download, the header takes
        /// >   priority. (If `Content-Disposition: inline`, Firefox prefers the header while Chrome
        /// >   prefers download.)
        download

        /// The URL that the hyperlink points to. Links are not restricted to HTTP-based URLs —
        /// they can use any URL scheme supported by browsers:
        ///
        /// * Sections of a page with fragment URLs
        /// * Pieces of media files with media fragments
        /// * Telephone numbers with tel: URLs
        /// * Email addresses with mailto: URLs
        /// * While web browsers may not support other URL schemes, web sites can with
        ///   registerProtocolHandler()
        href

        /// Hints at the human language of the linked URL. No built-in functionality. Allowed values
        /// are the same as the global lang attribute.
        hreflang

        /// A space-separated list of URLs. When the link is followed, the browser will send POST
        /// requests with the body PING to the URLs. Typically for tracking.
        ping

        /// The relationship of the linked URL as space-separated link types.
        rel

        /// Where to display the linked URL, as the name for a browsing context (a tab, window, or
        /// <iframe>). The following keywords have special meanings for where to load the URL:
        ///
        /// * `_self`: the current browsing context. (Default)
        /// * `_blank`: usually a new tab, but users can configure browsers to open a new window
        ///   instead.
        /// * `_parent`: the parent browsing context of the current one. If no parent, behaves as
        ///   _self.
        /// * `_top`: the topmost browsing context (the "highest" context that’s an ancestor of the
        ///   current one). If no ancestors, behaves as _self.
        ///
        /// > Note: When using target, add rel="noreferrer noopener" to avoid exploitation of the
        /// window.opener API;
        ///
        /// > Note: Linking to another page with target="_blank" will run the new page in the same
        /// process as your page. If the new page executes JavaScript, your page's performance may
        /// suffer. This can also be avoided by using rel="noreferrer noopener".
        target

        /// Hints at the linked URL’s format with a MIME type. No built-in functionality.
        type_
    }
}

html_element! {
    /// The [HTML Abbreviation element (`<abbr>`)][mdn] represents an abbreviation or acronym; the
    /// optional [`title`][title] attribute can provide an expansion or description for the
    /// abbreviation.
    ///
    /// The title attribute has a specific semantic meaning when used with the <abbr> element; it
    /// must contain a full human-readable description or expansion of the abbreviation. This text
    /// is often presented by browsers as a tooltip when the mouse cursor is hovered over the
    /// element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/abbr
    /// [title]: https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes#attr-title
    <abbr>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML Bring Attention To element (`<b>`)][mdn] is used to draw the reader's attention to
    /// the element's contents, which are not otherwise granted special importance.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/b
    <b>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML Bidirectional Isolate element (`<bdi>`)][mdn] tells the browser's bidirectional
    /// algorithm to treat the text it contains in isolation from its surrounding text.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/bdi
    <bdi>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML Bidirectional Text Override element (`<bdo>`)][mdn] overrides the current
    /// directionality of text, so that the text within is rendered in a different direction.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/bdo
    <bdo>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }

    attributes {
        /// The direction in which text should be rendered in this element's contents. Possible
        /// values are:
        ///
        /// * `ltr`: Indicates that the text should go in a left-to-right direction.
        /// * `rtl`: Indicates that the text should go in a right-to-left direction.
        dir
    }
}

html_element! {
    /// The [HTML `<br>` element][mdn] produces a line break in text (carriage-return). It is useful
    /// for writing a poem or an address, where the division of lines is significant.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/br
    <br>

    categories {
        Flow, Phrasing
    }
}

html_element! {
    /// The [HTML Citation element (`<cite>`)][mdn] is used to describe a reference to a cited
    /// creative work, and must include the title of that work.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/cite
    <cite>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML `<code>` element][mdn] displays its contents styled in a fashion intended to
    /// indicate that the text is a short fragment of computer code.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/code
    <code>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML `<data>` element][mdn] links a given content with a machine-readable translation.
    /// If the content is time- or date-related, the [`<time>`][time] element must be used.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/data
    /// [time]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time
    <data>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }

    attributes {
        /// This attribute specifies the machine-readable translation of the content of the element.
        value
    }
}

html_element! {
    /// The [HTML Definition element (`<dfn>`)][mdn] is used to indicate the term being defined
    /// within the context of a definition phrase or sentence.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dfn
    <dfn>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing // no <dfn> can be a descendant
        }
    }
}

html_element! {
    /// The [HTML `<em>` element][mdn] marks text that has stress emphasis. The `<em>` element can
    /// be nested, with each level of nesting indicating a greater degree of emphasis.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em
    <em>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML `<i>` element][mdn] represents a range of text that is set off from the normal
    /// text for some reason. Some examples include technical terms, foreign language phrases, or
    /// fictional character thoughts. It is typically displayed in italic type.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/i
    <i>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML Keyboard Input element (`<kbd>`)][mdn] represents a span of inline text denoting
    /// textual user input from a keyboard, voice input, or any other text entry device.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/kbd
    <kbd>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML Mark Text element (`<mark>`)][mdn] represents text which is marked or highlighted
    /// for reference or notation purposes, due to the marked passage's relevance or importance in
    /// the enclosing context.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/mark
    <mark>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML `<q>` element][mdn]  indicates that the enclosed text is a short inline quotation.
    /// Most modern browsers implement this by surrounding the text in quotation marks.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q
    <q>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }

    attributes {
        /// The value of this attribute is a URL that designates a source document or message for
        /// the information quoted. This attribute is intended to point to information explaining
        /// the context or the reference for the quote.
        cite
    }
}

html_element! {
    /// The [HTML Ruby Base (`<rb>`) element][mdn] is used to delimit the base text component of
    /// a [`<ruby>`][ruby] annotation, i.e. the text that is being annotated.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rb
    /// [ruby]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby
    <rb>
}

html_element! {
    /// The [HTML Ruby Fallback Parenthesis (`<rp>`) element][mdn] is used to provide fall-back
    /// parentheses for browsers that do not support display of ruby annotations using the
    /// [`<ruby>`][ruby] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rp
    /// [ruby]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby
    <rp>
}

only_text_children! { <rp> }

html_element! {
    /// The [HTML Ruby Text (`<rt>`) element][mdn] specifies the ruby text component of a ruby
    /// annotation, which is used to provide pronunciation, translation, or transliteration
    /// information for East Asian typography. The `<rt>` element must always be contained within a
    /// [`<ruby>`][ruby] element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/rt
    /// [ruby]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby
    <rt>

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
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
    <rtc>

    children {
        tags {
            <rt>
        }
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML `<ruby>` element][mdn] represents a ruby annotation. Ruby annotations are for
    /// showing pronunciation of East Asian characters.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ruby
    <ruby>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML `<s>` element][mdn] renders text with a strikethrough, or a line through it. Use
    /// the `<s>` element to represent things that are no longer relevant or no longer accurate.
    /// However, `<s>` is not appropriate when indicating document edits; for that, use the
    /// [`<del>`][del] and [`<ins>`][ins] elements, as appropriate.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/s
    /// [del]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del
    /// [ins]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ins
    <s>

    categories {
        Phrasing, Flow
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML Sample Element (`<samp>`)][mdn] is used to enclose inline text which represents
    /// sample (or quoted) output from a computer program.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/samp
    <samp>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML `<small>` element][mdn] represents side-comments and small print, like copyright
    /// and legal text, independent of its styled presentation. By default, it renders text within
    /// it one font-size small, such as from `small` to `x-small`.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/small
    <small>

    categories {
        Flow, Phrasing
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML `<span>` element][mdn] is a generic inline container for phrasing content, which
    /// does not inherently represent anything. It can be used to group elements for styling
    /// purposes (using the [`class`][class] or [`id`][id] attributes), or because they share
    /// attribute values, such as [`lang`][lang].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span
    /// [class]: https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes#attr-class
    /// [id]: https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes#attr-id
    /// [lang]: https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes#attr-lang
    <span>

    categories {
        Flow, Phrasing
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML Strong Importance Element (`<strong>`)][mdn] indicates that its contents have
    /// strong importance, seriousness, or urgency. Browsers typically render the contents in bold
    /// type.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong
    <strong>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML Subscript element (`<sub>`)][mdn] specifies inline text which should be displayed
    /// as subscript for solely typographical reasons.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sub
    <sub>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML Superscript element (`<sup>`)][mdn] specifies inline text which is to be displayed
    /// as superscript for solely typographical reasons.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sup
    <sup>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML `<time>` element][mdn] represents a specific period in time.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time
    <time>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }

    attributes {
        /// This attribute indicates the time and/or date of the element and must be in one of the
        /// formats described below.
        datetime
    }
}

html_element! {
    /// The [HTML Unarticulated Annotation Element (`<u>`)][mdn] represents a span of inline text
    /// which should be rendered in a way that indicates that it has a non-textual annotation.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/u
    <u>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML Variable element (`<var>`)][mdn] represents the name of a variable in a
    /// mathematical expression or a programming context.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/var
    <var>

    categories {
        Flow, Phrasing, Palpable
    }

    children {
        categories {
            Phrasing
        }
    }
}

html_element! {
    /// The [HTML `<wbr>` element][mdn] represents a word break opportunity—a position within text
    /// where the browser may optionally break a line, though its line-breaking rules would not
    /// otherwise create a break at that location.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/wbr
    <wbr>

    categories {
        Flow, Phrasing
    }
}

html_element! {
    /// The [HTML `<del>` element][mdn] represents a range of text that has been deleted from a
    /// document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/del
    <del>

    categories {
        Phrasing, Flow
    }

    attributes {
        /// A URI for a resource that explains the change (for example, meeting minutes).
        cite

        /// This attribute indicates the time and date of the change and must be a valid date string
        /// with an optional time. If the value cannot be parsed as a date with an optional time
        /// string, the element does not have an associated time stamp. For the format of the string
        /// without a time, see Date strings. The format of the string if it includes both date and
        /// time is covered in Local date and time strings.
        datetime
    }
}

html_element! {
    /// The [HTML `<ins>` element][mdn] represents a range of text that has been added to a
    /// document.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ins
    <ins>

    categories {
        Phrasing, Flow
    }

    attributes {
        /// A URI for a resource that explains the change (for example, meeting minutes).
        cite

        /// This attribute indicates the time and date of the change and must be a valid date string
        /// with an optional time. If the value cannot be parsed as a date with an optional time
        /// string, the element does not have an associated time stamp. For the format of the string
        /// without a time, see Date strings. The format of the string if it includes both date and
        /// time is covered in Local date and time strings.
        datetime
    }
}
