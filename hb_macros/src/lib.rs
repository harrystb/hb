use proc_macro::TokenStream;
use quote::quote;
use syn::fold::{self, Fold};
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, parse_quote, ItemFn, LitStr, Expr, ExprTry, Ident};
use proc_macro2::Span;

struct Msg {
    m: LitStr,
}

impl Msg {
    fn new(m: LitStr) -> Self {
        Msg {m: m}
    } 
}

impl Fold for Msg {
    fn fold_expr(&mut self, e: Expr) -> Expr {
        match e {
            Expr::Try(etry) => {
                let ex = etry.expr;
                let m = &self.m;
                let q = etry.question_token;
                parse_quote!({
                    #ex.map_err(|er| er.make_inner().msg(#m) )#q
                })
            },
            _ => fold::fold_expr(self, e),
        }
    }
    // stmt -> Local...
}


#[proc_macro_attribute]
pub fn context(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as ItemFn);

    // Build the output, possibly using quasi-quotation
    let mut message = Msg::new(parse_macro_input!(args as LitStr));

    let output = message.fold_item_fn(input);
    // Hand the output tokens back to the compiler
    TokenStream::from(quote!(#output))
}

#[proc_macro_attribute]
pub fn context2(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut input = parse_macro_input!(input as ItemFn);
    input.attrs.retain(|x| x.path.segments.last().map_or_else(|| true, |a| a.ident != Ident::new("context2", Span::call_site())));
    let block = input.block;
    let message = parse_macro_input!(args as LitStr);
    input.block = parse_quote!{
        {
            (||{#block})().map_err(|er| er.make_inner().msg(#message))
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(quote!{#input})
}