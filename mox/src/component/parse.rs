use {
    super::*,
    proc_macro2::Ident,
    quote::quote_spanned,
    syn::{
        braced, parenthesized,
        parse::{Parse, ParseStream, Result as ParseResult},
        parse2 as parse,
        punctuated::Punctuated,
        spanned::Spanned,
        visit_mut::VisitMut,
        Attribute, Block, Field, FieldsNamed, FnArg, FnDecl, Generics, ItemFn, ReturnType, Token,
        Type, Visibility, WhereClause, WherePredicate,
    },
};

fn parse_macro(input: ParseStream) -> ParseResult<ComponentMacro> {
    let attrs = input.call(Attribute::parse_outer)?;
    let vis: Visibility = input.parse()?;

    let fn_token: Token![fn] = input.parse()?;

    let ident: Ident = input.parse()?;
    let generics: Generics = input.parse()?;

    let content;
    let paren_token = parenthesized!(content in input);
    let inputs = content.parse_terminated(FnArg::parse)?;

    let (mut fields, mut field_names): (
        Punctuated<Field, Token![,]>,
        Punctuated<Ident, Token![,]>,
    ) = (Punctuated::new(), Punctuated::new());

    for arg in &inputs {
        let mut field: FieldsNamed = parse(quote_spanned!(arg.span()=> { #arg })).unwrap();
        let field: Field = field.named.pop().unwrap().into_value();
        field_names.push(field.ident.clone().unwrap());
        fields.push(field);
    }

    let mut where_clause: Option<WhereClause> = input.parse()?;

    let mut dependencies = Punctuated::new();

    if let Some(mut clause) = where_clause.as_mut() {
        let mut passthrough = Punctuated::new();

        let special_bound: Type = parse(quote::quote!(Self)).unwrap();
        for pred in &clause.predicates {
            match pred {
                WherePredicate::Type(ty_pred) if ty_pred.bounded_ty == special_bound => {
                    dependencies = ty_pred.bounds.clone();
                }
                p => {
                    passthrough.push(p.clone());
                }
            }
        }

        clause.predicates = passthrough;
    }

    dependencies.push(parse(quote::quote!(moxie::Runtime))?);

    let decl = Box::new(FnDecl {
        fn_token,
        paren_token,
        inputs,
        output: ReturnType::Default,
        variadic: None,
        generics: Generics {
            where_clause,
            ..generics
        },
    });

    let content;
    let brace_token = braced!(content in input);
    let stmts = content.call(Block::parse_within)?;

    let block = Box::new(Block { brace_token, stmts });

    let name = Name::new(parse(quote::quote!(#ident)).unwrap());

    let comp_fn = ItemFn {
        attrs,
        vis,
        constness: None,
        unsafety: None,
        asyncness: None,
        abi: None,
        ident: name.fn_name(),
        decl,
        block,
    };

    let mut comp_macro = ComponentMacro {
        name,
        comp_fn,
        fields,
        field_names,
        dependencies,
    };

    let mut threader = comp_macro.threader();
    threader.visit_item_fn_mut(&mut comp_macro.comp_fn);

    Ok(comp_macro)
}

impl Parse for ComponentMacro {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        parse_macro(input)
    }
}
