use hb_parse::error::ParseError;
use hb_parse::{CommonParserFunctions, SourceEmpty, StrParser};
use std::io;

fn main() {
    println!("Welcome to Parsex.");
    loop {
        println!("Please enter a string to try to parse. (Use \\n for a newline)");
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).ok();
        let raw_source = buffer.trim().replace("\\n", "\n");
        let mut parser = StrParser::new(&raw_source);
        println!("Please enter commands to parse the string. For a list of available commands, enter 'commands' or '?'");
        loop {
            buffer.clear();
            io::stdin().read_line(&mut buffer).ok();
            match buffer.trim().to_lowercase().as_str() {
                "" => break,
                "?" => {
                    println!(
                        r#"Select a parsing method to see the output (enter the command).
word   - read a word
string - read a string
int    - read an integer
uint   - read a unsigned integer
float  - read a float
symbol - read a symbol

A empty command will allow a new string to be entered."#
                    );
                }
                "commands" => {
                    println!(
                        r#"Select a parsing method to see the output (enter the command).
word   - read a word
string - read a string
int    - read an integer
uint   - read a unsigned integer
float  - read a float
symbol - read a symbol

A empty command will allow a new string to be entered."#
                    );
                }
                "word" => match parser.parse_word() {
                    Err(e) => println!("{}", e),
                    Ok(w) => println!("Word: {}", w),
                },
                "symbol" => match parser.parse_symbol() {
                    Err(e) => println!("{}", e),
                    Ok(w) => println!("Symbol: {}", w),
                },
                "string" => match parser.parse_string() {
                    Err(e) => println!("{}", e),
                    Ok(w) => println!("String: {}", w),
                },
                "int" => match parser.parse_num::<i64>() {
                    Err(e) => println!("{}", e),
                    Ok(w) => println!("Integer: {}", w),
                },
                "uint" => match parser.parse_num::<u64>() {
                    Err(e) => println!("{}", e),
                    Ok(w) => println!("Unsigned Integer: {}", w),
                },
                "float" => match parser.parse_float::<f64>() {
                    Err(e) => println!("{}", e),
                    Ok(w) => println!("Float: {}", w),
                },
                s => println!("invalid command '{}'", s),
            };
        }
    }
}
