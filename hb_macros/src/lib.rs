use proc_macro::TokenStream;
use quote::quote;
use syn::fold::{self, Fold};
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, parse_quote, ItemFn, LitStr, Expr, ExprTry, ReturnType, Signature, Type};
use proc_macro2::Span;

struct Msg {
    m: LitStr,
    rettype: Box<Type>,
}

impl Msg {
    fn new(m: LitStr, rettype: Box<Type>) -> Self {
        Msg {m, rettype}
    } 
}

impl Fold for Msg {
    fn fold_expr(&mut self, e: Expr) -> Expr {
        match e {
            Expr::Return(mut rexpr) => {
                match rexpr.expr {
                    Some(ex) => {
                        let m = &self.m;
                        let rettype = &self.rettype;
                        rexpr.expr = Some(parse_quote!(hb_parse::error::ConvertInto::<#rettype>::convert(#ex).map_err(|er| er.make_inner().msg(#m))));
                    },
                    None => (),
                }
                fold::fold_expr(self, Expr::Return(rexpr))
            },
            Expr::Try(mut texpr) => {
                let ex = texpr.expr;
                let m = &self.m;
                let rettype = &self.rettype;
                texpr.expr = parse_quote!(hb_parse::error::ConvertInto::<#rettype>::convert(#ex).map_err(|er| er.make_inner().msg(#m)));
                Expr::Try(texpr)
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
    let mut message;
    if let ReturnType::Type(_, r) = &input.sig.output {
        message = Msg::new(parse_macro_input!(args as LitStr), r.clone());
    } else {
        return TokenStream::from(quote!{#input});
    }
    let mut output = message.fold_item_fn(input.clone());
    let block = output.block.clone();
    let msg = message.m.clone();
    let rettype = message.rettype.clone();
    output.block = parse_quote!{
        {
            let ret: #rettype = {#block};
            ret.map_err(|er| er.make_inner().msg(#msg))
        }
    };
    TokenStream::from(quote!{#output})
}