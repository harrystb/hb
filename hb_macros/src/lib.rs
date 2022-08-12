use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::fold::{self, Fold};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Expr::Match;
use syn::{
    parse_macro_input, parse_quote, Attribute, Expr, ExprMatch, Field, Fields, FieldsNamed,
    FieldsUnnamed, Ident, ItemEnum, ItemFn, ItemStruct, LitStr, ReturnType, Token, Type, Variant,
};

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

// The Fold trait is used to inject the error handling into the the source code.
// The Fold will change this function...
// #[context("context message")]
// fn basic_exampleerror() -> Result<(), ExampleError> {
//     if io_error()? {
//         return example_error().map_err(|e| e.msg("msgs are great"));
//     }
//     example_error()
// }
//
// into...
// fn basic_exampleerror() -> Result<(), ExampleError> {
//     if hb_error::ConvertInto::Result<(), ExampleError>::convert(io_error()).map_err(|er| er.make_inner().msg("context message")? {
//         return hb_error::ConvertInto::Result<(), ExampleError>::convert(example_error().map_err(|e| e.msg("msgs are great"))).map_err(|er| er.make_inner().msg("context message");
//     }
//     example_error()
// }
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
/// This macro will change the following function...
/// #[context("context message")]
/// fn basic_exampleerror() -> Result<(), ExampleError> {
///     if io_error()? {
///         return example_error().map_err(|e| e.msg("msgs are great"));
///     }
///     example_error()
/// }
///
/// into...
/// fn basic_exampleerror() -> Result<(), ExampleError> {
///     #[allow(unreachable_code)]
///     let ret: Result<(), ExampleError> = {
///            #[warn(unreachable_code)]
///         if hb_error::ConvertInto::Result<(), ExampleError>::convert(io_error()).map_err(|er| er.make_inner().msg("context message")? {
///             return hb_error::ConvertInto::Result<(), ExampleError>::convert(example_error().map_err(|e| e.msg("msgs are great"))).map_err(|er| er.make_inner().msg("context message");
///         }
///         example_error()
///     };
///     #[allow(unreachable_code)]
///     ret.map_err(|er| e.make_inner().msg("context message")
/// }
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
    let mut output = message.fold_item_fn(input);
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

// The Fold trait is used to inject the error handling into the the source code.
// The Fold will change this function...
// #[convert_error]
// fn basic_exampleerror() -> Result<(), ExampleError> {
//     if io_error()? {
//         return example_error().map_err(|e| e.msg("msgs are great"));
//     }
//     example_error()
// }
//
// into...
// fn basic_exampleerror() -> Result<(), ExampleError> {
//     if hb_error::ConvertInto::Result<(), ExampleError>::convert(io_error())? {
//         return hb_error::ConvertInto::Result<(), ExampleError>::convert(example_error().map_err(|e| e.msg("msgs are great")));
//     }
//     example_error()
// }
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
/// This macro will change the following function...
/// #[convert_error]
/// fn basic_exampleerror() -> Result<(), ExampleError> {
///     if io_error()? {
///         return example_error().map_err(|e| e.msg("msgs are great"));
///     }
///     example_error()
/// }
///
/// into...
/// fn basic_exampleerror() -> Result<(), ExampleError> {
///     #[allow(unreachable_code)]
///     let ret: Result<(), ExampleError> = {
///            #[warn(unreachable_code)]
///         if hb_error::ConvertInto::Result<(), ExampleError>::convert(io_error())? {
///             return hb_error::ConvertInto::Result<(), ExampleError>::convert(example_error());
///         }
///         example_error()
///     };
///     #[allow(unreachable_code)]
///     ret
/// }
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
    let output = message.fold_item_fn(input);
    // Convert the SyntaxTree back into a TokenTree
    TokenStream::from(quote! {#output})
}

#[proc_macro_attribute]
pub fn hberror(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let mut output_struct = input.clone();
    let ident = input.ident;
    let identSource = Ident::new(&format!("{}Source", ident), Span::call_site());
    let temp_itemfn: ItemFn = parse_quote!(
        #[Source]
        fn dummy() -> () {}
    );
    let source_attr = temp_itemfn.attrs.iter().next().unwrap().clone();
    let temp_itemfn: ItemFn = parse_quote!(
        #[hberror]
        fn dummy() -> () {}
    );
    let hberror_attr = temp_itemfn.attrs.iter().next().unwrap().clone();
    let vis = input.vis;
    let mut attrs: Vec<Attribute> = input
        .attrs
        .iter()
        .filter(|a| **a != hberror_attr)
        .map(|a| a.clone())
        .collect();
    let mut enum_variants = syn::punctuated::Punctuated::<Variant, Comma>::new();
    let mut has_source_enum = false;
    match &input.fields {
        Fields::Named(namedfields) => {
            namedfields
                .named
                .iter()
                .filter(|f| f.attrs.contains(&source_attr))
                .map(|f| {
                    let ty = &f.ty;
                    enum_variants.push(Variant {
                        ident: f.ident.clone().unwrap(),
                        attrs: vec![],
                        fields: Fields::Unnamed(parse_quote!((#ty))),
                        discriminant: None,
                    });
                    has_source_enum = true;
                    ()
                })
                .collect::<Vec<()>>();
        }
        Fields::Unnamed(_) => (),
        Fields::Unit => (),
    };
    let mut enum_dislay_match = ExprMatch {
        attrs: vec![],
        match_token: Default::default(),
        expr: Box::new(parse_quote!(self)),
        brace_token: Default::default(),
        arms: vec![],
    };
    let mut variant_iter = enum_variants.iter();
    while let Some(variant) = variant_iter.next() {
        let ty = variant.ident.clone();
        enum_dislay_match
            .arms
            .push(parse_quote!(ty(e) => write!(f, "source error #ty...{}", e)))
    }
    /*    output_struct.fields = match output_struct.fields {
            Fields::Named(namedfields) => {
                let mut output_fields_punctuated = Punctuated::<Field, Token![,]>::new();
                namedfields
                    .named
                    .iter()
                    .map(|val| {
                        if !val.attrs.contains(&source_attr) {
                            let mut out_val = val.clone();
                            out_val.attrs = out_val
                                .attrs
                                .iter()
                                .filter(|a| **a != source_attr)
                                .map(|a| a.clone())
                                .collect();
                            output_fields_punctuated.push(out_val)
                        }
                    })
                    .collect::<Vec<()>>();
                output_fields_punctuated.push(Field {
                    attrs: vec![],
                    vis: vis.clone(),
                    ident: parse_quote!(msg),
                    colon_token: Some(Token![:](Span::call_site())),
                    ty: parse_quote!(String),
                });
                output_fields_punctuated.push(Field {
                    attrs: vec![],
                    vis: vis.clone(),
                    ident: parse_quote!(inner_msgs),
                    colon_token: Some(Token![:](Span::call_site())),
                    ty: parse_quote!(Vec<String>),
                });
                if has_source_enum {
                    output_fields_punctuated.push(Field {
                        attrs: vec![],
                        vis: vis.clone(),
                        ident: parse_quote!(source),
                        colon_token: Some(Token![:](Span::call_site())),
                        ty: parse_quote!(#identSource),
                    });
                }

                Fields::Named(parse_quote!({ output_fields_punctuated }))
            }
            Fields::Unit => {
                let mut output_fields_punctuated = Punctuated::<Field, Token![,]>::new();
                output_fields_punctuated.push(Field {
                    attrs: vec![],
                    vis: vis.clone(),
                    ident: parse_quote!(msg),
                    colon_token: Some(Token![:](Span::mixed_site())),
                    ty: parse_quote!(String),
                });
                output_fields_punctuated.push(Field {
                    attrs: vec![],
                    vis: vis.clone(),
                    ident: parse_quote!(inner_msgs),
                    colon_token: Some(Token![:](Span::mixed_site())),
                    ty: parse_quote!(Vec<String>),
                });
                if has_source_enum {
                    output_fields_punctuated.push(Field {
                        attrs: vec![],
                        vis: vis.clone(),
                        ident: parse_quote!(source),
                        colon_token: Some(Token![:](Span::mixed_site())),
                        ty: parse_quote!(#identSource),
                    });
                }
                Fields::Named(parse_quote!({ output_fields_punctuated }))
            }
            Fields::Unnamed(_) => {
                panic!("hberror macro does not work with structs with unnamed fields")
            }
        };
    */
    let output = match has_source_enum {
        true => {
            quote!(
                #output_struct

                #vis enum #identSource {
                    #enum_variants
                }

                impl Display for #identSource {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                        #enum_dislay_match
                    }
                }
            )
        }
        false => {
            quote!(
                #output_struct
            )
        }
    };
    output.into()
}
