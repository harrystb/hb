# hb_error

This crate holds the contains macros and traits to make creation of error types and handling of errors better.

The main features of this crate are the hberror and the context macros.

## hberror Macro
This macro is used to generate all of the boiler plate code for error types so that they all have the same format.
# Overview
This macro is applied to structs like this one.
 ```
 #[hberror]
 struct ExampleError {
 }
 ```
 This macro will modify the struct to add in the msg and inner_msgs fields as well as doing the impls for new, ErrorContext, Display and Debug.
 This will generate the following code:

 ```

 pub struct ExampleError {
     msg: String,
     inner_msgs: Vec<String>,
 }

 impl ExampleError {
     pub fn new() -> ExampleError {
         ExampleError {
             msg: default::Default(),
             inner_msgs: default::Default(),
         }
     }
 }

 impl ErrorContext for ExampleError {
     fn make_inner(mut self) -> ExampleError {
         self.inner_msgs.push(self.msg);
         self.msg = String::new();
         self
     }

     fn msg<T: Into<String>>(mut self, msg: T) -> ExampleError {
         self.msg = msg.into();
         self
     }
 }

 impl std::fmt::Display for ExampleError {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
         write!(f,"{}{}", self.msg, self.inner_msgs.join("\n...because..."))
     }
 }

 impl std::fmt::Debug for ExampleError {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
         write!(f,"{}{}", self.msg, self.inner_msgs.join("\n...because..."))
     }
 }
 ```
 # Custom fields and messages
 This macro also has the capability of dealing with custom fields and custom messages.
 Using the Default attribute on these custom fields will tell the macro to use the val provided
 in the new function rather than default::Default().

 ```
 #[hberror("{self.custom_field}:{self.msg}{self.inner_msgs.join(\"\n...due to...\")}")]
 struct ExampleError {
     #[Default(10)]
     custom_field: i32,
 }
 ```
 This becomes
 ```
 pub struct ExampleError {
     msg: String,
     inner_msgs: Vec<String>,
     custom_field: i32
 }

 impl ExampleError {
     pub fn new() -> ExampleError {
         ExampleError {
             msg: default::Default(),
             inner_msgs: default::Default(),
             custom_field: 10
         }
     }
 }

 impl ErrorContext for ExampleError {
     fn make_inner(mut self) -> ExampleError {
         self.inner_msgs.push(self.msg);
         self.msg = String::new();
         self
     }

     fn msg<T: Into<String>>(mut self, msg: T) -> ExampleError {
         self.msg = msg.into();
         self
     }
 }

 impl std::fmt::Display for ExampleError {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
         write!(f,"{}:{}{}", self.custom_field, self.msg, self.inner_msgs.join("\n...due to..."))
     }
 }

 impl std::fmt::Debug for ExampleError {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
         write!(f,"{}:{}{}", self.custom_field, self.msg, self.inner_msgs.join("\n...due to..."))
     }
 }
 ```
 # Easy Conversion from other errors
 You can also define errors that you want to convert from and store as a source variable which
 will be printed after message. The Source attribute tell the macro which fields to process as
 the source. A enum will be created using the identity of the sturct with Source at the end
 (eg ExampleErrorSource). This enum will have variants for each field marked with a source
 attribute as well as a None value. The variants for the sources are created using the field
 identity as the enum variant identity and contains the error type provided.

 This means that the context or convert_error macros can be used on this error type without any 
 additional work.
 ```
 #[hberror]
 struct ExampleError {
     #[Source]
     IOError: std::io::Error,
 }
 ```
 This becomes:
 ```
 use std::default;
 pub struct ExampleError {
     msg: String,
     inner_msgs: Vec<String>,
     source: ExampleErrorSource,
 }

 impl ExampleError {
     pub fn new() -> ExampleError {
         ExampleError {
             msg: default::Default(),
             inner_msgs: default::Default(),
             source: default::Default(),
         }
     }

     pub fn source(mut self, s: ExampleErrorSource) -> ExampleError {
         self.source = s;
         self
     }
 }

 impl ErrorContext for ExampleError {
     fn make_inner(mut self) -> ExampleError {
         self.inner_msgs.push(self.msg);
         self.msg = String::new();
         self
     }

     fn msg<T: Into<String>>(mut self, msg: T) -> ExampleError {
         self.msg = msg.into();
         self
     }
 }

 impl std::fmt::Display for ExampleError {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
         write!(f,"{}{}{}", self.msg, self.inner_msgs.join("\n...due to..."), self.source)
     }
 }

 impl std::fmt::Debug for ExampleError {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
         write!(f,"{}{}{}", self.msg, self.inner_msgs.join("\n...due to..."), self.source)
     }
 }

 pub enum ExampleErrorSource {
     None,
     IOError(std::io::Error),
 }

 impl std::default::Default for ExampleErrorSource {
     fn default() -> Self { ExampleErrorSource::None }
 }

 impl std::fmt::Display for ExampleErrorSource {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
         match self {
             ExampleErrorSource::None => Ok(()),
             ExampleErrorSource::IOError(e) => write!("\n...Source Error...{}", e)
         }
     }
 }

 impl std::fmt::Debug for ExampleErrorSource {
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
         match self {
             ExampleErrorSource::None => Ok(()),
             ExampleErrorSource::IOError(e) => write!("\n...Source Error...{}", e)
         }
     }
 }

 ```

## convert Macro
 Converts Errors returned by the function into the correct type for the
 function as well adding a context message provided. This requires the *From*
 to be implemented from the source Error to the return type Error.
 This macro will change the following function...
 ```
 #[context("context message")]
 fn basic_exampleerror() -> Result<(), ExampleError> {
     if io_error()? {
         return example_error().map_err(|e| e.msg("msgs are great"));
     }
     example_error()
 }
 ```
 into...
 ```
 fn basic_exampleerror() -> Result<(), ExampleError> {
     #[allow(unreachable_code)]
     let ret: Result<(), ExampleError> = {
            #[warn(unreachable_code)]
         if hb_error::ConvertInto::Result<(), ExampleError>::convert(io_error()).map_err(|er| er.make_inner().msg("context message")? {
             return hb_error::ConvertInto::Result<(), ExampleError>::convert(example_error().map_err(|e| e.msg("msgs are great"))).map_err(|er| er.make_inner().msg("context message");
         }
         example_error()
     };
     #[allow(unreachable_code)]
     ret.map_err(|er| e.make_inner().msg("context message")
 }
 ```
