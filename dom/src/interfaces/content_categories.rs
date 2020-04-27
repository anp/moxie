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
    Base, ExternalResourceLink, Meta, Style, Title, NoScript, Script,
}

/// Elements belonging to the flow content category typically contain text or
/// embedded content.
pub trait FlowContent: Node {}
mass_bare_impl! {
    FlowContent for
    Anchor, Abbreviation, Address, Article, Aside, Audio, BringAttentionTo, BidirectionalOverride,
    BlockQuote, LineBreak, Button, Canvas, Citation, Code, Data, DataList, Deleted, Details,
    DescriptionDefinition, Div, DescriptionList, Emphasis, Embed, FieldSet, Figure, Footer, Form,
    H1, H2, H3, H4, H5, H6, Header, HGroup, HorizontalRule, Italic, InlineFrame, Image, Input,
    Inserted, KeyboardInput, Label, Main, Map, Mark, Menu, Meter, Nav, NoScript, Object,
    OrderedList, Output, Paragraph, Picture, Preformatted, Progress, Quotation, Ruby, Strikethrough,
    Sample, Script, Section, Select, Small, Span, Strong, Subscript, Superscript, Table,
    Template, TextArea, Time, UnorderedList, Variable, Video, WordBreakOpportunity, Text,
    Area, // if it is a descendant of a <map> element
    ExternalResourceLink, // if the itemprop attribute is present
    Meta, // if the itemprop attribute is present
}

/// Elements belonging to the sectioning content model create a section in the
/// current outline that defines the scope of <header> elements, <footer>
/// elements, and heading content.
pub trait SectioningContent: Node {}
mass_bare_impl! {
    SectioningContent for
    Article, Aside, Nav, Section,
}

/// Heading content defines the title of a section, whether marked by an
/// explicit sectioning content element, or implicitly defined by the heading
/// content itself.
pub trait HeadingContent: Node {}
mass_bare_impl! {
    HeadingContent for
    H1, H2, H3, H4, H5, H6, HGroup,
}

/// Phrasing content defines the text and the mark-up it contains. Runs of
/// phrasing content make up paragraphs.
pub trait PhrasingContent: Node {}
mass_bare_impl! {
    PhrasingContent for
    Abbreviation, Audio, BringAttentionTo, BidirectionalOverride, LineBreak, Button, Canvas,
    Citation, Code, Data, DataList, DescriptionDefinition, Emphasis, Embed, Italic, InlineFrame,
    Image, Input, KeyboardInput, Label, Mark, Meter, NoScript, Object, Output, Picture, Progress,
    Quotation, Ruby, Sample, Script, Select, Small, Span, Strong, Subscript, Superscript, TextArea,
    Time, Variable, Video, WordBreakOpportunity, Text,
    Anchor, // if it contains only phrasing content
    Area, // if it is a descendant of a <map> element
    Deleted, // if it contains only phrasing content
    Inserted, // if it contains only phrasing content
    ExternalResourceLink, // if the itemprop attribute is present
    Map, // if it contains only phrasing content
    Meta, // if the itemprop attribute is present
}

/// Embedded content imports another resource or inserts content from another
/// mark-up language or namespace into the document.
pub trait EmbeddedContent: Node {}
mass_bare_impl! {
    EmbeddedContent for
    Audio, Canvas, Embed, InlineFrame, Image, Object, Picture, Video,
}

/// Interactive content includes elements that are specifically designed for
/// user interaction.
pub trait InteractiveContent: Node {}
mass_bare_impl! {
    InteractiveContent for
    Anchor, Button, Details, Embed, InlineFrame, Label, Select, TextArea,
    Audio, // if the controls attribute is present
    Image, // if the usemap attribute is present
    Input, // if the type attribute is not in the hidden state
    Menu, // if the type attribute is in the toolbar state
    Object, // if the usemap attribute is present
    Video, // if the controls attribute is present
}

/// Form-associated content comprises elements that have a form owner, exposed
/// by a form attribute. A form owner is either the containing <form> element or
/// the element whose id is specified in the form attribute.
pub trait FormAssociatedContent: Node {}
mass_bare_impl! {
    FormAssociatedContent for
    Button, FieldSet, Input, Label, Meter, Object, Output, Progress, Select, TextArea,
}

/// Elements that are listed in the form.elements and fieldset.elements IDL
/// collections.
pub trait ListedFormContent: FormAssociatedContent {}
mass_bare_impl! {
    ListedFormContent for
    Button, FieldSet, Input, Object, Output, Select, TextArea,
}

/// Elements that can be associated with <label> elements.
pub trait LabelableFormContent: FormAssociatedContent {}
mass_bare_impl! {
    LabelableFormContent for
    Button, Input, Meter, Output, Progress, Select, TextArea,
}

/// Elements that can be used for constructing the form data set when the form
/// is submitted.
pub trait SubmittableFormContent: FormAssociatedContent {}
mass_bare_impl! {
    SubmittableFormContent for
    Button, Input, Object, Select, TextArea,
}

/// Elements that can be affected when a form is reset.
pub trait ResettableFormContent: FormAssociatedContent {}
mass_bare_impl! {
    ResettableFormContent for
    Input, Output, Select, TextArea,
}
