use crate::ast::{Attribute, Element, For, If, IfClause, IfClauseTest, Node, Struct, Text};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::Ident;

impl Parse for Struct {
    fn parse(input: ParseStream) -> Result<Self> {
        let start_input_len = input.to_string().len();
        let name = input.parse()?;
        let generics = input.parse()?;
        let fields = {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::token::Paren) {
                let fields;
                syn::parenthesized!(fields in input);
                Punctuated::<(syn::Ident, syn::Type), syn::token::Comma>::parse_separated_nonempty_with(
                    &fields,
                    |inner| {
                        let name = inner.parse()?;
                        let _: syn::Token![:] = inner.parse()?;
                        let ty = inner.parse()?;
                        Ok((name, ty))
                    },
                )?.into_pairs()
                .map(|pair| pair.into_value())
                .collect()
            } else {
                Vec::new()
            }
        };
        let where_clause = if input.peek(syn::token::Where) {
            Some(input.parse()?)
        } else {
            None
        };
        let mut nodes = Vec::new();
        let inner;
        syn::braced!(inner in input);
        while !inner.is_empty() {
            nodes.push(inner.parse()?);
        }
        // We use the length of the tokens that define this template as a rough estimate of the
        // number of bytes the output of this template will occupy.
        // Lifted from Maud [1].
        // [1]: https://github.com/lfairy/maud/blob/13a5cfcaa31b3f6e2deb015ea49ef87d285cef7c/maud_macros/src/lib.rs#L38-L40
        let size_hint = start_input_len - input.to_string().len();
        Ok(Struct {
            name,
            generics,
            where_clause,
            fields,
            nodes,
            size_hint,
        })
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            Ok(Node::Element(input.parse()?))
        } else if lookahead.peek(syn::Token![@]) {
            let _: syn::Token![@] = input.parse()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::token::If) {
                let _: syn::token::If = input.parse()?;
                Ok(Node::If(input.parse()?))
            } else if lookahead.peek(syn::token::For) {
                let _: syn::token::For = input.parse()?;
                Ok(Node::For(input.parse()?))
            } else {
                Err(lookahead.error())
            }
        } else {
            Ok(Node::Text(input.parse()?))
        }
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let name = name.to_string();
        let mut id = None;
        let mut classes = Vec::new();

        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::Token![#]) {
                let _: syn::Token![#] = input.parse()?;
                id = Some(identifier_or_string_literal_or_expression(input)?);
            } else if lookahead.peek(syn::Token![.]) {
                let _: syn::Token![.] = input.parse()?;
                classes.push(identifier_or_string_literal_or_expression(input)?);
            } else {
                break;
            }
        }

        let attributes = {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::token::Bracket) {
                let attributes;
                syn::bracketed!(attributes in input);
                Punctuated::<Attribute, syn::Token![,]>::parse_terminated(&attributes)?
                    .into_pairs()
                    .map(|a| a.into_value())
                    .collect()
            } else {
                Vec::new()
            }
        };
        let (children, close) = {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::token::Semi) {
                let _: syn::Token![;] = input.parse()?;
                (Vec::new(), false)
            } else if lookahead.peek(syn::token::Brace) {
                let children;
                syn::braced!(children in input);
                (children.parse::<Many<_>>()?.0, true)
            } else {
                (Vec::new(), true)
            }
        };
        Ok(Element {
            name,
            id,
            classes,
            attributes,
            children,
            close,
        })
    }
}

impl Parse for Text {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::LitStr) {
            Ok(Text::String(input.parse::<syn::LitStr>()?.value()))
        } else if lookahead.peek(syn::token::Brace) {
            let inner;
            syn::braced!(inner in input);
            Ok(Text::Expr(inner.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for If {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut clauses = Vec::new();
        clauses.push(input.parse()?);
        let mut default = None;
        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::token::Else) {
                let _: syn::token::Else = input.parse()?;
                let lookahead = input.lookahead1();
                if lookahead.peek(syn::token::If) {
                    let _: syn::token::If = input.parse()?;
                    clauses.push(input.parse()?);
                } else {
                    default = {
                        let default;
                        syn::braced!(default in input);
                        Some(default.parse::<Many<_>>()?.0)
                    };
                    break;
                }
            } else {
                break;
            }
        }
        Ok(If { clauses, default })
    }
}

impl Parse for IfClause {
    fn parse(input: ParseStream) -> Result<Self> {
        let test = input.parse()?;
        let consequent = {
            let consequent;
            syn::braced!(consequent in input);
            consequent.parse::<Many<_>>()?.0
        };
        Ok(IfClause { test, consequent })
    }
}

impl Parse for IfClauseTest {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Let) {
            let _: syn::token::Let = input.parse()?;
            let pattern = input.parse()?;
            let _: syn::Token![=] = input.parse()?;
            let expr = input.parse()?;
            Ok(IfClauseTest::Let(pattern, expr))
        } else {
            Ok(IfClauseTest::Expr(input.parse()?))
        }
    }
}

impl Parse for For {
    fn parse(input: ParseStream) -> Result<Self> {
        let pat = input.parse()?;
        let _: syn::token::In = input.parse()?;
        let expr = input.parse()?;
        let body;
        syn::braced!(body in input);
        let body = body.parse::<Many<_>>()?.0;
        Ok(For { pat, expr, body })
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = identifier_or_string_literal_or_expression(input)?;
        let bool = {
            let lookahead = input.lookahead1();
            if lookahead.peek(syn::Token![?]) {
                let _: syn::Token![?] = input.parse()?;
                true
            } else {
                false
            }
        };
        let _: syn::Token![=] = input.parse()?;
        let value = input.parse()?;
        Ok(Attribute { name, value, bool })
    }
}

#[derive(Debug)]
pub struct Many<P>(pub Vec<P>);

impl<P: Parse> Parse for Many<P> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }
        Ok(Many(items))
    }
}

fn identifier_or_string_literal_or_expression(input: ParseStream) -> Result<syn::Expr> {
    let lookahead = input.lookahead1();
    let ident = input.step(|cursor| {
        if let Some((ident, rest)) = cursor.ident() {
            Ok((ident, rest))
        } else {
            Err(cursor.error(format!("expected identifier")))
        }
    });
    if let Ok(ident) = ident {
        let string = ident.to_string();
        Ok(syn::parse_quote!(#string))
    } else if lookahead.peek(syn::LitStr) {
        let string = input.parse::<syn::LitStr>()?.value();
        Ok(syn::parse_quote!(#string))
    } else if lookahead.peek(syn::token::Brace) {
        let inner;
        syn::braced!(inner in input);
        Ok(inner.parse()?)
    } else {
        Err(lookahead.error())
    }
}
