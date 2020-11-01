//! The elements here are used to create and handle tabular data.

html_element! {
    /// The [HTML Table Caption element (`<caption>`)][mdn] specifies the caption (or title) of a
    /// table, and if used is *always* the first child of a [`<table>`][table].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/caption
    /// [table]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table
    <caption>

    children {
        categories {
            Flow
        }
    }
}

html_element! {
    /// The [HTML `<col>` element][mdn] defines a column within a table and is used for defining
    /// common semantics on all common cells. It is generally found within a [`<colgroup>`][cg]
    /// element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/col
    /// [cg]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/colgroup
    <col>

    attributes {
        /// This attribute contains a positive integer indicating the number of consecutive columns
        /// the `<col>` element spans. If not present, its default value is 1.
        span
    }
}

html_element! {
    /// The [HTML `<colgroup>` element][mdn] defines a group of columns within a table.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/colgroup
    <colgroup>

    children {
        tags {
            <col> // if the span attribute is present
        }
    }

    attributes {
        /// This attribute contains a positive integer indicating the number of consecutive columns
        /// the `<colgroup>` element spans. If not present, its default value is 1.
        ///
        /// > Note: This attribute is applied on the attributes of the column group, it has no
        /// > effect on the CSS styling rules associated with it or, even more, to the cells of the
        /// > column's members of the group.
        /// >
        /// > The span attribute is not permitted if there are one or more `<col>` elements within
        /// > the `<colgroup>`.
        span
    }
}

html_element! {
    /// The [HTML `<table>` element][mdn] represents tabular data — that is, information presented
    /// in a two-dimensional table comprised of rows and columns of cells containing data.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table
    <table>

    categories {
        Flow
    }

    children {
        tags {
            <caption>, <colgroup>, <thead>, <tbody>, <tr>, <tfoot>
        }
    }
}

html_element! {
    /// The [HTML Table Body element (`<tbody>`)][mdn] encapsulates a set of table rows
    /// ([`<tr>`][tr] elements), indicating that they comprise the body of the table
    /// ([`<table>`][table]).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tbody
    /// [tr]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr
    /// [table]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table
    <tbody>

    children {
        tags {
            <tr>
        }
    }
}

html_element! {
    /// The [HTML `<td>` element][mdn] defines a cell of a table that contains data. It participates
    /// in the *table model*.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td
    <td>

    categories {
        Sectioning
    }

    children {
        categories {
            Flow
        }
    }

    attributes {
        /// This attribute contains a non-negative integer value that indicates for how many columns
        /// the cell extends. Its default value is 1. Values higher than 1000 will be considered as
        /// incorrect and will be set to the default value (1).
        colspan

        /// This attribute contains a list of space-separated strings, each corresponding to the id
        /// attribute of the `<th>` elements that apply to this element.
        headers

        /// This attribute contains a non-negative integer value that indicates for how many rows
        /// the cell extends. Its default value is 1; if its value is set to 0, it extends until the
        /// end of the table section (`<thead>`, `<tbody>`, `<tfoot>`, even if implicitly defined),
        /// that the cell belongs to. Values higher than 65534 are clipped down to 65534.
        rowspan
    }
}

html_element! {
    /// The [HTML `<tfoot>` element][mdn] defines a set of rows summarizing the columns of the
    /// table.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tfoot
    <tfoot>

    children {
        tags {
            <tr>
        }
    }
}

html_element! {
    /// The [HTML `<th>` element][mdn] defines a cell as header of a group of table cells. The exact
    /// nature of this group is defined by the [`scope`][scope] and [`headers`][headers] attributes.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th
    /// [scope]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th#attr-scope
    /// [headers]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th#attr-headers
    <th>

    children {
        categories {
            Flow // no header, footer, sectioning, or heading descendants
        }
    }

    attributes {
        /// This attribute contains a short abbreviated description of the cell's content. Some
        /// user-agents, such as speech readers, may present this description before the content
        /// itself.
        abbr

        /// This attribute contains a non-negative integer value that indicates for how many columns
        /// the cell extends. Its default value is 1. Values higher than 1000 will be considered as
        /// incorrect and will be set to the default value (1).
        colspan

        /// This attribute contains a list of space-separated strings, each corresponding to the id
        /// attribute of the `<th>` elements that apply to this element.
        headers

        /// This attribute contains a non-negative integer value that indicates for how many rows
        /// the cell extends. Its default value is 1; if its value is set to 0, it extends until the
        /// end of the table section (`<thead>`, `<tbody>`, `<tfoot>`, even if implicitly defined),
        /// that the cell belongs to. Values higher than 65534 are clipped down to 65534.
        rowspan

        /// This enumerated attribute defines the cells that the header (defined in the `<th>`)
        /// element relates to. It may have the following values:
        ///
        /// * `row`: The header relates to all cells of the row it belongs to.
        /// * `col`: The header relates to all cells of the column it belongs to.
        /// * `rowgroup`: The header belongs to a rowgroup and relates to all of its cells. These
        ///   cells can be placed to the right or the left of the header, depending on the value of
        ///   the dir attribute in the `<table>` element.
        /// * `colgroup`: The header belongs to a colgroup and relates to all of its cells.
        /// * `auto`
        ///
        /// The default value when this attribute is not specified is auto.
        scope
    }
}

html_element! {
    /// The [HTML `<thead>` element][mdn] defines a set of rows defining the head of the columns of
    /// the table.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/thead
    <thead>

    children {
        tags {
            <tr>
        }
    }
}

html_element! {
    /// The [HTML `<tr>` element][mdn] defines a row of cells in a table. The row's cells can then
    /// be established using a mix of [`<td>`][td] (data cell) and [`<th>`][th] (header cell)
    /// elements.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr
    /// [td]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td
    /// [th]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th
    <tr>

    children {
        tags {
            <td>, <th>
        }
        categories {
            ScriptSupporting
        }
    }
}
