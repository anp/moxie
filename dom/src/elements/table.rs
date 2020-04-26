//! The elements here are used to create and handle tabular data.

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

html_element! {
    /// The [HTML Table Caption element (`<caption>`)][mdn] specifies the caption (or title) of a
    /// table, and if used is *always* the first child of a [`<table>`][table].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/caption
    /// [table]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table
    caption -> TableCaption
}

html_element! {
    /// The [HTML `<col>` element][mdn] defines a column within a table and is used for defining
    /// common semantics on all common cells. It is generally found within a [`<colgroup>`][cg]
    /// element.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/col
    /// [cg]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/colgroup
    col -> TableColumn
}

html_element! {
    /// The [HTML `<colgroup>` element][mdn] defines a group of columns within a table.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/colgroup
    colgroup -> TableColumnGroup
}

html_element! {
    /// The [HTML `<table>` element][mdn] represents tabular data — that is, information presented
    /// in a two-dimensional table comprised of rows and columns of cells containing data.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table
    table -> Table
}

html_element! {
    /// The [HTML Table Body element (`<tbody>`)][mdn] encapsulates a set of table rows
    /// ([`<tr>`][tr] elements), indicating that they comprise the body of the table
    /// ([`<table>`][table]).
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tbody
    /// [tr]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr
    /// [table]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table
    tbody -> TableBody
}

html_element! {
    /// The [HTML `<td>` element][mdn] defines a cell of a table that contains data. It participates
    /// in the *table model*.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td
    td -> TableCell
}

html_element! {
    /// The [HTML `<tfoot>` element][mdn] defines a set of rows summarizing the columns of the
    /// table.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tfoot
    tfoot -> TableFooter
}

html_element! {
    /// The [HTML `<th>` element][mdn] defines a cell as header of a group of table cells. The exact
    /// nature of this group is defined by the [`scope`][scope] and [`headers`][headers] attributes.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th
    /// [scope]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th#attr-scope
    /// [headers]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th#attr-headers
    th -> TableHeaderCell
}

html_element! {
    /// The [HTML `<thead>` element][mdn] defines a set of rows defining the head of the columns of
    /// the table.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/thead
    thead -> TableHeader
}

html_element! {
    /// The [HTML `<tr>` element][mdn] defines a row of cells in a table. The row's cells can then
    /// be established using a mix of [`<td>`][td] (data cell) and [`<th>`][th] (header cell)
    /// elements.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr
    /// [td]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td
    /// [th]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th
    tr -> TableRow
}
