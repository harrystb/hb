use hb_error::*;
struct ExampleError {
    msg: String,
    inner_errors: Vec<String>,
}

#[hberror]
struct AnotherExampleError {}

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

    let exerror = AnotherExampleError::new()
        .msg("first msg")
        .make_inner()
        .msg("second msg");
    println!(
        "Another Exmaple 1: Trying out the macro to fill in the contents of an error type\n{}\n",
        exerror
    );
}
