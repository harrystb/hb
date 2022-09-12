use hb_error::*;
struct ExampleError {
    msg: String,
    inner_errors: Vec<String>,
}

impl ExampleError {
    pub fn new() -> ExampleError {
        ExampleError {
            msg: String::new(),
            inner_errors: vec![],
        }
    }
}

//Implement Context so context macro can be used
impl hb_error::ErrorContext for ExampleError {
    fn make_inner(mut self) -> Self {
        self.inner_errors.push(self.msg);
        self.msg = String::new();
        self
    }

    fn msg<T: Into<String>>(mut self, msg: T) -> Self {
        self.msg = msg.into();
        self
    }
}

//Implement Display/Debug for printing
impl std::fmt::Display for ExampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}\n{}", self.msg, self.inner_errors.join("\n"))
    }
}

impl std::fmt::Debug for ExampleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}\n{}", self.msg, self.inner_errors.join("\n"))
    }
}

//Implement From<io::Error> to allow ConvertInto for the convert/context macros
impl From<std::io::Error> for ExampleError {
    fn from(_: std::io::Error) -> ExampleError {
        ExampleError {
            msg: "encountered an IO Error".to_string(),
            inner_errors: vec![],
        }
    }
}

//Some functions to generate errors for the examples
fn io_error() -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "Example IO Error.",
    ))
}

fn example_error() -> Result<(), ExampleError> {
    Err(ExampleError::new().msg("Generated ExampleError."))
}

// Example #1
// Add context onto a fall through Err object of type ExampleError
// The fall through Err before the macro adds context will be:
// ExampleError {
//   msg: "Generated ExampleError.",
//   inner_error: vec![]
// }
//
// The err returned from this function (ie after the context macro changes it) will be:
// ExampleError {
//   msg: "additional context - basic example",
//   inner_error: vec!["Generated ExampleError."]
// }
#[context("addition context - basic example")]
fn basic_exampleerror() -> Result<(), ExampleError> {
    example_error()
}

// Example #2
// Add context onto a try expression (ie ? operator) on a Err object of type ExampleError
// The Err that would be returned by the ? before the macro adds context will be:
// ExampleError {
//   msg: "Generated ExampleError.",
//   inner_error: vec![]
// }
//
// The err returned from this function (ie after the context macro changes it) will be:
// ExampleError {
//   msg: "additional context - basic example",
//   inner_error: vec!["Generated ExampleError."]
// }
#[context("addition context - basic example")]
fn basic2_exampleerror() -> Result<(), ExampleError> {
    example_error()?;
    Ok(())
}

// Example #3
// Add context onto a return expression on a Err object of type ExampleError
// The Err that would be returned by the return expression before the macro adds context will be:
// ExampleError {
//   msg: "Generated ExampleError.",
//   inner_error: vec![]
// }
//
// The err returned from this function (ie after the context macro changes it) will be:
// ExampleError {
//   msg: "additional context - basic example",
//   inner_error: vec!["Generated ExampleError."]
// }
#[context("addition context - basic example")]
fn basic3_exampleerror() -> Result<(), ExampleError> {
    return example_error();
}

// Example #4
// Convert from an io::Error into a ExampleError
// The io::Error before conversion will be
// std::io::Error::new(
//     std::io::ErrorKind::AlreadyExists,
//     "Example IO Error.",
// )
//
// after conversion the ExampleError will be:
// ExampleError {
//   msg: "encountered an IO Error",
//   inner_error: vec![]
// }
#[convert_error]
fn basic4_exampleerror() -> Result<(), ExampleError> {
    return io_error();
}

// Example #5
// Convert from an io::Error into a ExampleError and add context
// The io::Error before conversion will be
// std::io::Error::new(
//     std::io::ErrorKind::AlreadyExists,
//     "Example IO Error.",
// )
//
// after conversion but before the context is added the ExampleError will be:
// ExampleError {
//   msg: "encountered an IO Error",
//   inner_error: vec![]
// }
//
// The final error will be:
// ExampleError {
//   msg: "additional context - basic example" ,
//   inner_error: vec!["encountered an IO Error"]
// }
#[context("addition context - basic example")]
fn basic5_exampleerror() -> Result<(), ExampleError> {
    return io_error();
}

// Example #6
// The context macro also allows parameters to be in the message using {}
#[context("add parameters '{p}' - basic example")]
fn basic6_exampleerror(p: &str) -> Result<(), ExampleError> {
    return io_error();
}

// The macro hberror will generate the boilerplate of Errors that are used in this style.
// The #[Source] attribute tells the hberror macro to implement a source struct (called
// AnotherExampleErrorSource in the example below) which also means that the From<> traits will be
// implemented
//
// The below Struct will be transformed into:
//    struct AnotherExampleError {
//        msg: String,
//        inner_msgs: Vec<String>,
//        source: AnotherExampleErrorSource,
//    }
//
//    impl AnotherExampleError {
//        fn new() -> AnotherExampleError {
//            AnotherExampleError {
//                msg: String::new(),
//                inner_msgs: vec![],
//                source: AnotherExampleErrorSource::None,
//            }
//        }
//
//        fn source(mut self, s: AnotherExampleErrorSource) -> AnotherExampleError {
//            self.source = s;
//            self
//        }
//    }
//
//    impl ErrorContext for AnotherExampleError {
//        fn make_inner(mut self) -> AnotherExampleError {
//            self.inner_msgs.push(self.msg);
//            self.msg = String::new();
//            self
//        }
//
//        fn msg<T: Into<String>>(mut self, msg: T) -> AnotherExampleError {
//            self.msg = msg.into();
//            self
//        }
//    }
//
//    impl std::fmt::Display for AnotherExampleError {
//        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
//            write!(f, "{}\n...because... {}{}", self.msg, self.inner_msgs.join("\n...because... "), self.source)
//        }
//    }
//
//    impl std::fmt::Debug for AnotherExampleError {
//        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
//            write!(f, "{}\n...because... {}{}", self.msg, self.inner_msgs.join("\n...because... "), self.source)
//        }
//    }
//
//    enum AnotherExampleErrorSource {
//        IOError(std::io::Error),
//        None,
//    }
//
//    impl std::fmt::Display for AnotherExampleErrorSource {
//        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
//            match self {
//                AnotherExampleErrorSource::IOError(e) => write!(f, "\nsource error {}...{}",stringify!(IOError), e),
//                None => Ok(()),
//        }
//    }
//
//    impl std::fmt::Debug for AnotherExampleErrorSource {
//        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
//                AnotherExampleErrorSource::IOError(e) => write!(f, "\nsource error {}...{}",stringify!(IOError), e),
//                None => Ok(()),
//        }
//    }
#[hberror("{self.val}{self.msg}{self.inner_msgs.join(\"\n...due to...\")}")]
struct AnotherExampleError {
    #[Default(10)]
    val: i32,
    #[Source]
    IOError: std::io::Error,
}

#[context("hberror generated example from io error")]
fn more_exampleerror() -> Result<(), AnotherExampleError> {
    return io_error();
}

#[context("adding a second layer of context")]
fn more_exampleerror2() -> Result<(), AnotherExampleError> {
    return more_exampleerror();
}

fn main() {
    println!(
        "Basic Example 1: Adding context to a fall through error\n{}\n",
        basic_exampleerror().err().unwrap()
    );
    println!(
        "Basic Example 2: Adding context to a try expression (? operator)\n{}\n",
        basic2_exampleerror().err().unwrap()
    );
    println!(
        "Basic Example 3: Adding context to a returned error\n{}\n",
        basic3_exampleerror().err().unwrap()
    );
    println!(
        "Basic Example 4: Converting an io::Error into a ExampleError\n{}\n",
        basic4_exampleerror().err().unwrap()
    );
    println!(
        "Basic Example 5: Adding converting and adding context with an io:Error\n{}\n",
        basic5_exampleerror().err().unwrap()
    );
    println!(
        "Basic Example 6: Adding converting and adding context with an io:Error\n{}\n",
        basic6_exampleerror("val").err().unwrap()
    );

    println!(
        "Another Example 1: Trying out the macro to fill in the contents of an error type\n{}\n",
        more_exampleerror().err().unwrap()
    );
    println!(
        "Another Example 2: Adding a layer of context on top\n{}\n",
        more_exampleerror2().err().unwrap()
    );
}
