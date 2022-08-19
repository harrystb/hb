use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, TokenStreamExt};
use syn::fold::{self, Fold};
use syn::parse::{Parse, Parser};
use syn::punctuated::Pair::Punctuated;
use syn::token::Comma;
use syn::Expr::Match;
use syn::{
    parse_macro_input, parse_quote, Attribute, Expr, ExprBlock, ExprMatch, Field, Fields,
    FieldsNamed, FieldsUnnamed, Ident, ImplItem, Item, ItemEnum, ItemFn, ItemStruct, LitStr,
    ReturnType, Token, Type, Variant,
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
pub fn hberror(args: TokenStream, input: TokenStream) -> TokenStream {
    // parse the input as an ItemStruct, it should panic if anything other than a struct is annotated with this macro
    let input = parse_macro_input!(input as ItemStruct);
    // create a copy which we can edit easily
    let mut output_struct = input.clone();

    // get some details from the struct
    // Struct Name
    let ident = input.ident;
    // Create the source enum's name from the structs name
    let identSource = Ident::new(&format!("{}Source", ident), Span::call_site());
    // visibility of the struct - eg pub
    let vis = input.vis;

    // Calculate some constants for use with matching
    let temp_itemfn: ItemFn = parse_quote!(
        #[Source]
        fn dummy() -> () {}
    );
    let source_attr = temp_itemfn.attrs.iter().next().unwrap().clone();
    let mut enum_variants = syn::punctuated::Punctuated::<Variant, Comma>::new();
    let mut non_source_fields = syn::punctuated::Punctuated::<Field, Comma>::new();
    let mut source_from_impl_items: Vec<Item> = vec![];
    let mut has_source_enum = false;
    match &input.fields {
        Fields::Named(namedfields) => {
            namedfields
                .named
                .iter()
                .filter(|f| f.attrs.contains(&source_attr))
                .for_each(|f| {
                    let ty = &f.ty;
                    let enum_ident = f.ident.clone().unwrap();
                    enum_variants.push(Variant {
                        ident: f.ident.clone().unwrap(),
                        attrs: vec![],
                        fields: Fields::Unnamed(parse_quote!((#ty))),
                        discriminant: None,
                    });
                    has_source_enum = true;
                    source_from_impl_items.push(parse_quote!(impl From<#ty> for #ident {
                        fn from(e: #ty) -> #ident {
                            #ident::new().source(#identSource::#enum_ident(e))
                        }
                    }));
                });
            namedfields
                .named
                .iter()
                .filter(|f| !f.attrs.contains(&source_attr))
                .for_each(|f| non_source_fields.push(f.clone()));
        }
        Fields::Unnamed(_) => (),
        Fields::Unit => (),
    };
    let mut enum_display_match = ExprMatch {
        attrs: vec![],
        match_token: Default::default(),
        expr: Box::new(parse_quote!(self)),
        brace_token: Default::default(),
        arms: vec![],
    };
    for variant in &enum_variants {
        let ty = variant.ident.clone();
        enum_display_match
            .arms
            .push(parse_quote!(#identSource::#ty(e) => write!(f, "\nsource error {}...{}",stringify!(#ty), e)));
    }
    if has_source_enum {
        enum_variants.push(Variant {
            ident: parse_quote!(None),
            attrs: vec![],
            fields: Fields::Unit,
            discriminant: None,
        });
        enum_display_match
            .arms
            .push(parse_quote!(#identSource::None => Ok(())))
    }
    let mut final_fields = Fields::Named(match has_source_enum {
        true => parse_quote! {
            {
                msg: String,
                inner_msgs: Vec<String>,
                source: #identSource,
            }
        },
        false => parse_quote! {
            {
                msg: String,
                inner_msgs: Vec<String>,
            }
        },
    });
    match &mut final_fields {
        Fields::Named(n) => non_source_fields
            .iter()
            .for_each(|f| n.named.push(f.clone())),
        _ => (),
    };
    let mut msg_args = syn::punctuated::Punctuated::<Expr, Comma>::new();
    msg_args.push(parse_quote!(f));
    match syn::parse::<LitStr>(args) {
        Err(e) => {
            if has_source_enum {
                msg_args.push(parse_quote!("{}{}{}"));
                msg_args.push(parse_quote!(self.msg));
                msg_args.push(parse_quote!(self.inner_msgs.join("\n...becuase...")));
                msg_args.push(parse_quote!(self.source));
            } else {
                msg_args.push(parse_quote!("{}{}"));
                msg_args.push(parse_quote!(self.msg));
                msg_args.push(parse_quote!(self.inner_msgs.join("\n...becuase...")));
            }
        }
        Ok(litstr) => {
            let str = litstr.value();
            let mut brace_contents: Vec<String> = vec![];
            let mut in_brace = false;
            let mut buf = String::new();
            let mut out_str = String::new();
            for c in str.chars() {
                if c == '{' {
                    in_brace = true;
                    out_str.push(c);
                } else if c == '}' {
                    if buf.len() > 0 {
                        brace_contents.push(buf);
                        buf = String::new();
                    }
                    in_brace = false;
                    out_str.push(c);
                } else if in_brace {
                    buf.push(c);
                } else {
                    out_str.push(c);
                }
            }

            for content in brace_contents {
                msg_args
                    .push(syn::parse_str::<Expr>(&content).expect(
                        format!("cannot convert {} into an expression.", content).as_str(),
                    ));
            }
            msg_args.insert(1, parse_quote!(#out_str));
        }
    }

    output_struct.fields = final_fields;
    let output = match has_source_enum {
        true => {
            let mut out = quote!(
                #output_struct

                impl #ident {
                    #vis fn new() -> #ident {
                        #ident {
                            msg: String::new(),
                            inner_msgs: vec![],
                            source: #identSource::None,
                        }
                    }

                    #vis fn source(mut self, s: #identSource) -> #ident {
                        self.source = s;
                        self
                    }
                }

                impl ErrorContext for #ident {
                    fn make_inner(mut self) -> #ident {
                        self.inner_msgs.push(self.msg);
                        self.msg = String::new();
                        self
                    }

                    fn msg<T: Into<String>>(mut self, msg: T) -> #ident {
                        self.msg = msg.into();
                        self
                    }
                }

                impl std::fmt::Display for #ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                        write!(#msg_args)
                    }
                }

                impl std::fmt::Debug for #ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                        write!(#msg_args)
                    }
                }

                #vis enum #identSource {
                    #enum_variants
                }

                impl std::fmt::Display for #identSource {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                        #enum_display_match
                    }
                }

                impl std::fmt::Debug for #identSource {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                        #enum_display_match
                    }
                }
            );
            out.append_all(source_from_impl_items);
            out
        }
        false => {
            let mut out = quote!(
                #output_struct

                impl #ident {
                    #vis fn new() -> #ident {
                        #ident {
                            msg: String::new(),
                            inner_msgs: vec![],
                        }
                    }
                }

                impl ErrorContext for #ident {
                    fn make_inner(mut self) -> #ident {
                        self.inner_msgs.push(self.msg);
                        self.msg = String::new();
                        self
                    }

                    fn msg<T: Into<String>>(mut self, msg: T) -> #ident {
                        self.msg = msg.into();
                        self
                    }
                }

                impl std::fmt::Display for #ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                        write!(#msg_args)
                    }
                }

                impl std::fmt::Debug for #ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                        write!(#msg_args)
                    }
                }
            );
            out
        }
    };
    output.into()
}
