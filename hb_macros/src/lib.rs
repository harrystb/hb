use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, TokenStreamExt};
use syn::fold::{self, Fold};
use syn::token::Comma;
use syn::{
    parse_macro_input, parse_quote, Attribute, Expr, ExprMatch, FieldValue, Fields, Ident, Item,
    ItemFn, ItemStruct, LitStr, ReturnType, Type, Variant,
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
                        if m.value().contains('{') {
                            let mut msg_args = syn::punctuated::Punctuated::<Expr, Comma>::new();
                            let str = m.value();
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
                                msg_args.push(
                                    syn::parse_str::<Expr>(&content).expect(
                                        format!("cannot convert {} into an expression.", content)
                                            .as_str(),
                                    ),
                                );
                            }
                            msg_args.insert(0, parse_quote!(#out_str));
                            rexpr.expr = Some(
                                parse_quote!(hb_error::ConvertInto::<#rettype>::convert(#ex).map_err(|er| er.make_inner().msg(format!(#msg_args)))),
                            );
                        } else {
                            rexpr.expr = Some(
                                parse_quote!(hb_error::ConvertInto::<#rettype>::convert(#ex).map_err(|er| er.make_inner().msg(#m))),
                            );
                        }
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
/// ```
/// #[context("context message")]
/// fn basic_exampleerror() -> Result<(), ExampleError> {
///     if io_error()? {
///         return example_error().map_err(|e| e.msg("msgs are great"));
///     }
///     example_error()
/// }
/// ```
/// into...
/// ```
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
/// ```
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

#[proc_macro_attribute]
pub fn context_doc(args: TokenStream, input: TokenStream) -> TokenStream {
    // convert the input TokenStream into a ItemFn syntax object
    let input = parse_macro_input!(input as ItemFn);
    let mut message;
    let doc_comments: Vec<Attribute> = input
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("doc"))
        .map(|a| a.clone())
        .collect();
    if doc_comments.len() == 0 || doc_comments.len() > 1 {
        panic!("context_doc only works with single line doc comments.")
    }
    //TODO: Look at how to parse doc comment
    let tokens = doc_comments.first().unwrap().tokens.clone();
    // Extract the return type from the function signature
    if let ReturnType::Type(_, r) = &input.sig.output {
        // Read the args provided as a LitStr ie contents of the () after context in the attibute
        // Then create a ContextMsg object
        message = ContextMsg::new(parse_macro_input!(tokens as LitStr), r.clone());
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
/// ```
/// #[convert_error]
/// fn basic_exampleerror() -> Result<(), ExampleError> {
///     if io_error()? {
///         return example_error().map_err(|e| e.msg("msgs are great"));
///     }
///     example_error()
/// }
/// ```
///
/// into...
/// ```
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
/// ```
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

/// This macro is used to generate all of the boiler plate code for error types so that they all
/// have the same format.
/// # Overview
/// This macro is applied to structs like this one.
/// ```
/// #[hberror]
/// struct ExampleError {
/// }
/// ```
/// This macro will modify the struct to add in the msg and inner_msgs fields as well as doing the impls for new, ErrorContext, Display and Debug.
/// This will generate the following code:
///
/// ```
///
/// pub struct ExampleError {
///     msg: String,
///     inner_msgs: Vec<String>,
/// }
///
/// impl ExampleError {
///     pub fn new() -> ExampleError {
///         ExampleError {
///             msg: default::Default(),
///             inner_msgs: default::Default(),
///         }
///     }
/// }
///
/// impl ErrorContext for ExampleError {
///     fn make_inner(mut self) -> ExampleError {
///         self.inner_msgs.push(self.msg);
///         self.msg = String::new();
///         self
///     }
///
///     fn msg<T: Into<String>>(mut self, msg: T) -> ExampleError {
///         self.msg = msg.into();
///         self
///     }
/// }
///
/// impl std::fmt::Display for ExampleError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
///         write!(f,"{}{}", self.msg, self.inner_msgs.join("\n...because..."))
///     }
/// }
///
/// impl std::fmt::Debug for ExampleError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
///         write!(f,"{}{}", self.msg, self.inner_msgs.join("\n...because..."))
///     }
/// }
/// ```
/// # Custom fields and messages
/// This macro also has the capability of dealing with custom fields and custom messages.
/// Using the Default attribute on these custom fields will tell the macro to use the val provided
/// in the new function rather than default::Default().
///
/// ```
/// #[hberror("{self.custom_field}:{self.msg}{self.inner_msgs.join(\"\n...due to...\")}")]
/// struct ExampleError {
///     #[Default(10)]
///     custom_field: i32,
/// }
/// ```
/// This becomes
/// ```
/// pub struct ExampleError {
///     msg: String,
///     inner_msgs: Vec<String>,
///     custom_field: i32
/// }
///
/// impl ExampleError {
///     pub fn new() -> ExampleError {
///         ExampleError {
///             msg: default::Default(),
///             inner_msgs: default::Default(),
///             custom_field: 10
///         }
///     }
/// }
///
/// impl ErrorContext for ExampleError {
///     fn make_inner(mut self) -> ExampleError {
///         self.inner_msgs.push(self.msg);
///         self.msg = String::new();
///         self
///     }
///
///     fn msg<T: Into<String>>(mut self, msg: T) -> ExampleError {
///         self.msg = msg.into();
///         self
///     }
/// }
///
/// impl std::fmt::Display for ExampleError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
///         write!(f,"{}:{}{}", self.custom_field, self.msg, self.inner_msgs.join("\n...due to..."))
///     }
/// }
///
/// impl std::fmt::Debug for ExampleError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
///         write!(f,"{}:{}{}", self.custom_field, self.msg, self.inner_msgs.join("\n...due to..."))
///     }
/// }
/// ```
/// # Easy Conversion from other errors
/// You can also define errors that you want to convert from and store as a source variable which
/// will be printed after message. The Source attribute tell the macro which fields to process as
/// the source. A enum will be created using the identity of the sturct with Source at the end
/// (eg ExampleErrorSource). This enum will have variants for each field marked with a source
/// attribute as well as a None value. The variants for the sources are created using the field
/// identity as the enum variant identity and contains the error type provided.
///
/// This means that the context or convert_error macros can be used on this error type without any
/// additional work.
/// ```
/// #[hberror]
/// struct ExampleError {
///     #[Source]
///     IOError: std::io::Error,
/// }
/// ```
/// This becomes:
/// ```
/// use std::default;
/// pub struct ExampleError {
///     msg: String,
///     inner_msgs: Vec<String>,
///     source: ExampleErrorSource,
/// }
///
/// impl ExampleError {
///     pub fn new() -> ExampleError {
///         ExampleError {
///             msg: default::Default(),
///             inner_msgs: default::Default(),
///             source: default::Default(),
///         }
///     }
///
///     pub fn source(mut self, s: ExampleErrorSource) -> ExampleError {
///         self.source = s;
///         self
///     }
/// }
///
/// impl ErrorContext for ExampleError {
///     fn make_inner(mut self) -> ExampleError {
///         self.inner_msgs.push(self.msg);
///         self.msg = String::new();
///         self
///     }
///
///     fn msg<T: Into<String>>(mut self, msg: T) -> ExampleError {
///         self.msg = msg.into();
///         self
///     }
/// }
///
/// impl std::fmt::Display for ExampleError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
///         write!(f,"{}{}{}", self.msg, self.inner_msgs.join("\n...due to..."), self.source)
///     }
/// }
///
/// impl std::fmt::Debug for ExampleError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
///         write!(f,"{}{}{}", self.msg, self.inner_msgs.join("\n...due to..."), self.source)
///     }
/// }
///
/// pub enum ExampleErrorSource {
///     None,
///     IOError(std::io::Error),
/// }
///
/// impl std::default::Default for ExampleErrorSource {
///     fn default() -> Self { ExampleErrorSource::None }
/// }
///
/// impl std::fmt::Display for ExampleErrorSource {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
///         match self {
///             ExampleErrorSource::None => Ok(()),
///             ExampleErrorSource::IOError(e) => write!("\n...Source Error...{}", e)
///         }
///     }
/// }
///
/// impl std::fmt::Debug for ExampleErrorSource {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
///         match self {
///             ExampleErrorSource::None => Ok(()),
///             ExampleErrorSource::IOError(e) => write!("\n...Source Error...{}", e)
///         }
///     }
/// }
///
/// ```
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
    let ident_source = Ident::new(&format!("{}Source", ident), Span::call_site());
    // visibility of the struct - eg pub
    let vis = input.vis;

    // sort and process the provided fields
    let mut has_source_enum = false;
    let mut enum_variants = syn::punctuated::Punctuated::<Variant, Comma>::new();
    let mut new_fields = syn::punctuated::Punctuated::<FieldValue, Comma>::new();
    let mut enum_display_match = ExprMatch {
        attrs: vec![],
        match_token: Default::default(),
        expr: Box::new(parse_quote!(self)),
        brace_token: Default::default(),
        arms: vec![],
    };
    let mut source_from_impl_items: Vec<Item> = vec![];
    let mut custom_fields = vec![];
    match &input.fields {
        Fields::Named(namedfields) => {
            namedfields.named.iter().for_each(|f| {
                let mut handled = false;
                for a in &f.attrs {
                    if a.path.is_ident("Source") {
                        let mut cleaned_field = f.clone();
                        let mut cleaned_attrs : Vec<Attribute> = vec![];
                        f.attrs
                            .iter()
                            .filter(|a| !a.path.is_ident("Source"))
                            .for_each(|a| cleaned_attrs.push(a.clone()));
                        cleaned_field.attrs = cleaned_attrs;
                        let ty = f.ty.clone();
                        let f_ident = f.ident.clone().unwrap();
                        enum_variants.push(Variant {
                            ident: f.ident.clone().unwrap(),
                            attrs: vec![],
                            fields: Fields::Unnamed(parse_quote!((#ty))),
                            discriminant: None,
                        });
                        enum_display_match
                            .arms
                            .push(parse_quote!(#ident_source::#f_ident(e) => write!(f, "\n...source error {}...{}",stringify!(#f_ident), e)));
                        source_from_impl_items.push(parse_quote!(impl From<#ty> for #ident {
                                fn from(e: #ty) -> #ident {
                                    #ident::new().source(#ident_source::#f_ident(e))
                                }
                            }));
                        has_source_enum = true;
                        handled = true;
                        break;
                    } else if a.path.is_ident("Default") {
                        let mut cleaned_field = f.clone();
                        let mut cleaned_attrs : Vec<Attribute> = vec![];
                        f.attrs
                            .iter()
                            .filter(|a| !a.path.is_ident("Default"))
                            .for_each(|a| cleaned_attrs.push(a.clone()));
                        cleaned_field.attrs = cleaned_attrs;
                        custom_fields.push((cleaned_field, Some(a.tokens.clone())));
                        handled = true;
                        break;
                    }
                }
                if !handled {
                    custom_fields.push((f.clone(), None));
                }
            });
        }
        Fields::Unnamed(_) => (),
        Fields::Unit => (),
    };
    if has_source_enum {
        enum_variants.push(Variant {
            ident: parse_quote!(None),
            attrs: vec![],
            fields: Fields::Unit,
            discriminant: None,
        });
        enum_display_match
            .arms
            .push(parse_quote!(#ident_source::None => Ok(())))
    }
    new_fields.push(parse_quote!(msg: String::new()));
    new_fields.push(parse_quote!(inner_msgs: vec![]));
    let mut final_fields = Fields::Named(match has_source_enum {
        true => {
            new_fields.push(parse_quote!(source: #ident_source::None));
            parse_quote! {
                {
                    msg: String,
                    inner_msgs: Vec<String>,
                    source: #ident_source,
                }
            }
        }
        false => parse_quote! {
            {
                msg: String,
                inner_msgs: Vec<String>,
            }
        },
    });
    match &mut final_fields {
        Fields::Named(ref mut named_fields) => {
            for (f, toks) in custom_fields {
                let f_ident = f.ident.clone();
                named_fields.named.push(f);
                match toks {
                    Some(tok) => new_fields.push(parse_quote!(#f_ident: #tok)),
                    None => new_fields.push(parse_quote!(#f_ident: Default::default())),
                }
            }
        }
        _ => panic!("should not happen"),
    }
    let mut msg_args = syn::punctuated::Punctuated::<Expr, Comma>::new();
    msg_args.push(parse_quote!(f));
    match syn::parse::<LitStr>(args) {
        Err(_) => {
            if has_source_enum {
                msg_args.push(parse_quote!("{}{}{}"));
                msg_args.push(parse_quote!(self.msg));
                msg_args.push(parse_quote!(self.inner_msgs.join("\n...because...")));
                msg_args.push(parse_quote!(self.source));
            } else {
                msg_args.push(parse_quote!("{}{}"));
                msg_args.push(parse_quote!(self.msg));
                msg_args.push(parse_quote!(self.inner_msgs.join("\n...because...")));
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
            if has_source_enum {
                msg_args.push(parse_quote!(self.source));
                out_str.push_str("{}")
            }
            msg_args.insert(1, parse_quote!(#out_str));
        }
    }
    output_struct.fields = final_fields;

    //Build the output code
    //Parts that don't matter if there is any source enum
    let main_output = quote!(
                #output_struct

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

    // Add source enum stuff if there is one
    let final_output = match has_source_enum {
        true => {
            let mut out = quote!(
                #main_output

                impl #ident {
                    /// Create a new error with empty values.
                    #vis fn new() -> #ident {
                        #ident {
                            #new_fields
                        }
                    }

                    /// Set the source value of the error type with special enum. This function is
                    /// usually used by the From implementation between the source error type and
                    /// the final error type, where the source error is stored in the applicable
                    /// variant of the enum in the source field of the error.
                    #vis fn source(mut self, s: #ident_source) -> #ident {
                        self.source = s;
                        self
                    }
                }

                /// This enum represents all of the errors that can be a source for this Error.
                /// The trait From has been implemented between all of these Error types and the
                /// main error type where the source field is set to the applicable variant of this
                /// enum.
                #vis enum #ident_source {
                    #enum_variants
                }

                impl std::fmt::Display for #ident_source {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                        #enum_display_match
                    }
                }

                impl std::fmt::Debug for #ident_source {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                        #enum_display_match
                    }
                }

                impl std::default::Default for #ident_source {
                    fn default() -> Self { #ident_source::None }
                }
            );
            out.append_all(source_from_impl_items);
            out
        }
        false => {
            parse_quote!(
                #main_output

                impl #ident {
                    #vis fn new() -> #ident {
                        #ident {
                            #new_fields
                        }
                    }
                }

            )
        }
    };
    final_output.into()
}
