//! Every HTML element is a member of one or more content categories — these
//! categories group elements that share common characteristics. This is a loose
//! grouping (it doesn't actually create a relationship among elements of these
//! categories), but they help define and describe the categories' shared
//! behavior and their associated rules, especially when you come upon their
//! intricate details. It's also possible for elements to not be a member of any
//! of these categories.
//!
//! There are three types of content categories:
//!
//! 1. Main content categories, which describe common rules shared by many
//!    elements.
//! 2. Form-related content categories, which describe rules common to
//!    form-related elements.
//! 3. Specific content categories, which describe rare categories shared
//!    only by a few elements, sometimes only in a specific context.

use crate::{
    elements::{
        embedding::*, forms::*, interactive::*, media::*, metadata::*, scripting::*, sectioning::*,
        table::*, text_content::*, text_semantics::*, Template,
    },
    memo_node::Text,
    prelude::*,
};

/// Elements belonging to the metadata content category modify the presentation
/// or the behavior of the rest of the document, set up links to other
/// documents, or convey other out of band information.
pub trait MetadataContent: Node {}
mass_bare_impl! {
    MetadataContent for
    <base>, <link>, <meta>, <noscript>, <script>, <style>, <title>
}

/// Elements belonging to the flow content category typically contain text or
/// embedded content.
pub trait FlowContent: Node {}
impl FlowContent for Text {}
mass_bare_impl! {
    FlowContent for
    <a>, <abbr>, <address>, <article>, <aside>, <audio>, <b>,<bdo>, <bdi>, <blockquote>, <br>,
    <button>, <canvas>, <cite>, <code>, <data>, <datalist>, <del>, <details>, <dfn>, <div>, <dl>,
    <em>, <embed>, <fieldset>, <figure>, <footer>, <form>, <h1>, <h2>, <h3>, <h4>, <h5>, <h6>,
    <header>, <hgroup>, <hr>, <i>, <iframe>, <img>, <input>, <ins>, <kbd>, <label>, <main>, <map>,
    <mark>, <menu>, <meter>, <nav>, <noscript>, <object>, <ol>, <output>, <p>, <picture>, <pre>,
    <progress>, <q>, <ruby>, <s>, <samp>, <script>, <section>, <select>, <small>, <span>, <strong>,
    <sub>, <sup>, <table>, <template>, <textarea>, <time>, <ul>, <var>, <video>,
    <wbr>,
    <area>, // if it is a descendant of a <map> element
    <link>, // if the itemprop attribute is present
    <meta>, // if the itemprop attribute is present
    <style> // if the scoped attribute is present
}

/// Elements belonging to the sectioning content model create a section in the
/// current outline that defines the scope of <header> elements, <footer>
/// elements, and heading content.
pub trait SectioningContent: Node {}
mass_bare_impl! {
    SectioningContent for
    <article>, <aside>, <nav>, <section>
}

/// Heading content defines the title of a section, whether marked by an
/// explicit sectioning content element, or implicitly defined by the heading
/// content itself.
pub trait HeadingContent: Node {}
mass_bare_impl! {
    HeadingContent for
    <h1>, <h2>, <h3>, <h4>, <h5>, <h6>, <hgroup>
}

/// Phrasing content defines the text and the mark-up it contains. Runs of
/// phrasing content make up paragraphs.
pub trait PhrasingContent: Node {}
impl PhrasingContent for Text {}
mass_bare_impl! {
    PhrasingContent for
    <abbr>, <audio>, <b>, <bdo>, <br>, <button>, <canvas>, <cite>, <code>, <data>, <datalist>,
    <dfn>, <em>, <embed>, <i>, <iframe>, <img>, <input>, <kbd>, <label>, <mark>, <meter>,
    <noscript>, <object>, <output>, <picture>, <progress>, <q>, <ruby>, <samp>, <script>, <select>,
    <small>, <span>, <strong>, <sub>, <sup>, <textarea>, <time>, <var>, <video>, <wbr>,
    <a>, // if it contains only phrasing content
    <area>, // if it is a descendant of a <map> element
    <del>, // if it contains only phrasing content
    <ins>, // if it contains only phrasing content
    <link>, // if the itemprop attribute is present
    <map>, // if it contains only phrasing content
    <meta> // if the itemprop attribute is present
}

/// Embedded content imports another resource or inserts content from another
/// mark-up language or namespace into the document.
pub trait EmbeddedContent: Node {}
mass_bare_impl! {
    EmbeddedContent for
    <audio>, <canvas>, <embed>, <iframe>, <img>, <object>, <picture>, <video>
}

/// Interactive content includes elements that are specifically designed for
/// user interaction.
pub trait InteractiveContent: Node {}
mass_bare_impl! {
    InteractiveContent for
    <a>, <button>, <details>, <embed>, <iframe>, <label>, <select>, <textarea>,
    <audio>, // if the controls attribute is present
    <img>, // if the usemap attribute is present
    <input>, // if the type attribute is not in the hidden state
    <menu>, // if the type attribute is in the toolbar state
    <object>, // if the usemap attribute is present
    <video> // if the controls attribute is present
}

/// Form-associated content comprises elements that have a form owner, exposed
/// by a form attribute. A form owner is either the containing <form> element or
/// the element whose id is specified in the form attribute.
pub trait FormAssociatedContent: Node {}
mass_bare_impl! {
    FormAssociatedContent for
    <button>, <fieldset>, <input>, <label>, <meter>, <object>, <output>, <progress>, <select>,
    <textarea>
}

/// Elements that are listed in the form.elements and fieldset.elements IDL
/// collections.
pub trait ListedFormContent: FormAssociatedContent {}
mass_bare_impl! {
    ListedFormContent for
    <button>, <fieldset>, <input>, <object>, <output>, <select>, <textarea>
}

/// Elements that can be associated with <label> elements.
pub trait LabelableFormContent: FormAssociatedContent {}
mass_bare_impl! {
    LabelableFormContent for
    <button>, <input>, <meter>, <output>, <progress>, <select>, <textarea>
}

/// Elements that can be used for constructing the form data set when the form
/// is submitted.
pub trait SubmittableFormContent: FormAssociatedContent {}
mass_bare_impl! {
    SubmittableFormContent for
    <button>, <input>, <object>, <select>, <textarea>
}

/// Elements that can be affected when a form is reset.
pub trait ResettableFormContent: FormAssociatedContent {}
mass_bare_impl! {
    ResettableFormContent for
    <input>, <output>,<select>, <textarea>
}
