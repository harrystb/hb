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
            Expr::Return(mut rexpr) => {
                match rexpr.expr {
                    Some(ex) => {
                        let m = &self.m;
                        rexpr.expr = Some(parse_quote!({
                            #ex.map_err(|er| Into::<ParseError>::into(er).make_inner().msg(#m) )
                        }));
                    },
                    None => (),
                }
                Expr::Return(rexpr)
            },
            _ => fold::fold_expr(self, e),
        }
    }
}

#[proc_macro_attribute]
pub fn context(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut input = parse_macro_input!(input as ItemFn);
    //input.attrs.retain(|x| x.path.segments.last().map_or_else(|| true, |a| a.ident != Ident::new("context2", Span::call_site())));
    let block = input.block.clone();
    let rettype = input.sig.output.clone();
    let mut message = Msg::new(parse_macro_input!(args as LitStr));
    let msg = message.m.clone();
    input.block = parse_quote!{
        {
            let ret = match {#block} {
                Err(e) => return Err(Into::<ParseError>::into(e).make_inner().msg(#msg)),
                Ok(o) => return Ok(o),
            }
        }
    };
    // Build the output, possibly using quasi-quotation

    //let output = message.fold_item_fn(input);

    // Hand the output tokens back to the compiler
    TokenStream::from(quote!{#input})
}