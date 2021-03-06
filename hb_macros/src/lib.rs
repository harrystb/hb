use proc_macro::TokenStream;
use quote::quote;
use syn::fold::{self, Fold};
use syn::{parse_macro_input, parse_quote, Expr, ItemFn, LitStr, ReturnType, Type};

/// Struct to handle the folding of the ItemFn.
/// Holds the return type and message for use by the fold functions.
struct ContextMsg {
    m: LitStr,
    rettype: Box<Type>,
}

impl ContextMsg {
    fn new(m: LitStr, rettype: Box<Type>) -> Self {
        ContextMsg { m, rettype }
    }
}

impl Fold for ContextMsg {
    fn fold_expr(&mut self, e: Expr) -> Expr {
        match e {
            Expr::Return(mut rexpr) => {
                match rexpr.expr {
                    Some(ex) => {
                        let m = &self.m;
                        let rettype = &self.rettype;
                        rexpr.expr = Some(
                            parse_quote!(hb_error::ConvertInto::<#rettype>::convert(#ex).map_err(|er| er.make_inner().msg(#m))),
                        );
                    }
                    None => (),
                }
                fold::fold_expr(self, Expr::Return(rexpr))
            }
            Expr::Try(mut texpr) => {
                let ex = texpr.expr;
                let m = &self.m;
                let rettype = &self.rettype;
                texpr.expr = parse_quote!(hb_error::ConvertInto::<#rettype>::convert(#ex).map_err(|er| er.make_inner().msg(#m)));
                Expr::Try(texpr)
            }
            _ => fold::fold_expr(self, e),
        }
    }
}

/// Converts Errors returned by the function into the correct type for the
/// function as well adding a context message provided.
#[proc_macro_attribute]
pub fn context(args: TokenStream, input: TokenStream) -> TokenStream {
    // convert the input TokenStream into a ItemFn syntax object
    let input = parse_macro_input!(input as ItemFn);
    let mut message;
    // Extract the return type from the function signature
    if let ReturnType::Type(_, r) = &input.sig.output {
        // Read the args provided as a LitStr ie contents of the () after context in the attibute
        // Then create a ContextMsg object
        message = ContextMsg::new(parse_macro_input!(args as LitStr), r.clone());
    } else {
        // If the return type is the default return type () then skip processing
        return TokenStream::from(quote! {#input});
    }
    // Handle Return and ? by folding the ItemFn syntax tree with the ContextMsg object
    let mut output = message.fold_item_fn(input.clone());
    // Wrap the context of the function to grab the fall through Result then add the context
    // onto the any errors with map_err
    let block = output.block.clone();
    let msg = message.m.clone();
    let rettype = message.rettype.clone();
    output.block = parse_quote! {
        {
            #[allow(unreachable_code)]
            let ret: #rettype = {
                #[warn(unreachable_code)]
                #block
            };
            #[allow(unreachable_code)]
            ret.map_err(|er| er.make_inner().msg(#msg))
        }
    };
    // Convert the SyntaxTree back into a TokenTree
    TokenStream::from(quote! {#output})
}

/// Struct to handle the folding of the ItemFn.
struct Converter {
    rettype: Box<Type>,
}

impl Fold for Converter {
    fn fold_expr(&mut self, e: Expr) -> Expr {
        match e {
            Expr::Return(mut rexpr) => {
                match rexpr.expr {
                    Some(ex) => {
                        let rettype = &self.rettype;
                        rexpr.expr =
                            Some(parse_quote!(hb_error::ConvertInto::<#rettype>::convert(#ex)));
                    }
                    None => (),
                }
                fold::fold_expr(self, Expr::Return(rexpr))
            }
            Expr::Try(mut texpr) => {
                let ex = texpr.expr;
                let rettype = &self.rettype;
                texpr.expr = parse_quote!(hb_error::ConvertInto::<#rettype>::convert(#ex));
                Expr::Try(texpr)
            }
            _ => fold::fold_expr(self, e),
        }
    }
}

/// Converts Errors returned by the function into the correct type for the
/// function as well adding a context message provided.
#[proc_macro_attribute]
pub fn convert_error(_args: TokenStream, input: TokenStream) -> TokenStream {
    // convert the input TokenStream into a ItemFn syntax object
    let input = parse_macro_input!(input as ItemFn);
    let mut message;
    // Extract the return type from the function signature
    if let ReturnType::Type(_, r) = &input.sig.output {
        // Read the args provided as a LitStr ie contents of the () after context in the attibute
        // Then create a ContextMsg object
        message = Converter { rettype: r.clone() };
    } else {
        // If the return type is the default return type () then skip processing
        return TokenStream::from(quote! {#input});
    }
    // Handle Return and ? by folding the ItemFn syntax tree with the ContextMsg object
    let output = message.fold_item_fn(input.clone());
    // Convert the SyntaxTree back into a TokenTree
    TokenStream::from(quote! {#output})
}
