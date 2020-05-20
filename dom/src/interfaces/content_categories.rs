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

use crate::interfaces::node::Node;

/// Elements belonging to the metadata content category modify the presentation
/// or the behavior of the rest of the document, set up links to other
/// documents, or convey other out of band information.
pub trait MetadataContent: Node {}

/// Elements belonging to the flow content category typically contain text or
/// embedded content.
pub trait FlowContent: Node {}

/// Elements belonging to the sectioning content model create a section in the
/// current outline that defines the scope of <header> elements, <footer>
/// elements, and heading content.
pub trait SectioningContent: Node {}

/// Heading content defines the title of a section, whether marked by an
/// explicit sectioning content element, or implicitly defined by the heading
/// content itself.
pub trait HeadingContent: Node {}

/// Phrasing content defines the text and the mark-up it contains. Runs of
/// phrasing content make up paragraphs.
pub trait PhrasingContent: Node {}

/// Embedded content imports another resource or inserts content from another
/// mark-up language or namespace into the document.
pub trait EmbeddedContent: Node {}

/// Interactive content includes elements that are specifically designed for
/// user interaction.
pub trait InteractiveContent: Node {}

/// Form-associated content comprises elements that have a form owner, exposed
/// by a form attribute. A form owner is either the containing <form> element or
/// the element whose id is specified in the form attribute.
pub trait FormAssociatedContent: Node {}

/// Elements that are listed in the form.elements and fieldset.elements IDL
/// collections.
pub trait ListedContent: Node {}

/// Elements that can be associated with <label> elements.
pub trait LabelableContent: Node {}

/// Elements that can be used for constructing the form data set when the form
/// is submitted.
pub trait SubmittableContent: Node {}

/// Elements that can be affected when a form is reset.
pub trait ResettableContent: Node {}

/// Content is palpable when it's neither empty or hidden; it is content that is
/// rendered and is substantive. Elements whose model is flow content or
/// phrasing content should have at least one node which is palpable.
pub trait PalpableContent: Node {}

/// Script-supporting elements are elements which don't directly contribute to
/// the rendered output of a document. Instead, they serve to support scripts,
/// either by containing or specifying script code directly, or by specifying
/// data that will be used by scripts.
pub trait ScriptSupportingContent: Node {}
