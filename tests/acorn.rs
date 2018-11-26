//! These tests were adapted from test cases used by https://github.com/RReverser/acorn-jsx.
//!
//! Copied under the MIT license.

use moxie::mox;

#[test]
fn lex_numlit() {
    let two = mox! { <div><br />7x invalid-js-identifier</div> };
}

#[test]
fn basic_a() {
    let _ = mox! { <a /> };
}

#[test]
fn name() {
    let _ = mox! { <a v /> };
}

#[test]
fn attr() {
    let _ = mox! { <a foo="bar"> {value} <b><c /></b></a> };
}

#[test]
fn interpolate_attr() {
    let _ = mox! { <a b={" "} c=" " d="&amp;" e="&ampr;" /> };
}

#[test]
fn newlines_in_elems() {
    let _ = mox! {
        <a
        />
    };
}

#[test]
fn unicode_name() {
    let _ = mox! { <日本語></日本語> };
}

#[test]
fn escaping() {
    let _ = mox! {
        <AbC_def
            test="&#x0026;&#38;">
            bar
            baz
        </AbC_def>
    };
}

#[test]
fn attr_interpolate() {
    let _ = mox! { <a b={if x { <c /> } { <d /> }} /> };
}

#[test]
fn simple_interpolate() {
    let _ = mox! { <a>{1}</a> };
}

#[test]
fn comment_interpolate() {
    let _ = mox! { <a>{// this is a comment
    "hello"}</a> };
}

#[test]
fn weird_text() {
    let _ = mox! { <div>@test content</div> };
}

#[test]
fn text_ident() {
    let _ = mox! { <div><br />7x invalid-js-identifier</div> };
}

#[test]
fn text_ident_experiment() {
    let _ = mox! { <div><br />7xinvalid-ident</div> };
}

#[test]
fn element_children() {
    let _ = mox! { <LeftRight left=<a /> right=<b>monkeys gorillas</b> /> };
}

#[test]
fn path_names() {
    let _ = mox! { <a::b></a::b> };
}

#[test]
fn deep_path_names() {
    let _ = mox! { <a::b::c></a::b::c> };
}

#[test]
fn nested_exprs() {
    let _ = mox! { <A aa={aa.bb.cc} bb={bb.cc.dd}><div>{aa.b}</div></A> };
}

#[test]
fn fragment() {
    let _ = mox! { <><div></div></> };
}

#[test]
fn nested_tags_with_text_siblings() {
    let _ = mox! { <p>foo <a href="test"> bar</a> baz</p> };
}

#[test]
fn expr_props_spread() {
    let _ = mox! { <div>{<div {...test} />}</div> };
}

#[test]
fn block_inside_interpolation() {
    let _ = mox! { <div>{ {a} }</div> };
}

#[test]
fn tricky_slash() {
    let _ = mox! { <div>/text</div> };
}

#[test]
fn sibling_interpolation() {
    let _ = mox! { <div>{a}{b}</div> };
}

#[test]
fn regression_weird_attr_value_str() {
    let _ = mox! { <path d="M230 80\n\t\tA 45 45, 0, 1, 0, 275 125 \r\n    L 275 80 Z"/> };
}
