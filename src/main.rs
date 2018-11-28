use std::error::Error;
use std::io::prelude::*;
use std::{fmt, io, process};

use csv;
use edn;
use edn::parser::Parser;

struct EdnPrinter<'a> {
    edn: &'a edn::Value,
}

impl<'a> EdnPrinter<'a> {
    fn new(edn: &'a edn::Value) -> Self {
        EdnPrinter { edn: edn }
    }
}

impl<'a> From<&'a edn::Value> for EdnPrinter<'a> {
    fn from(edn: &'a edn::Value) -> Self {
        EdnPrinter { edn: edn }
    }
}

impl<'a> fmt::Display for EdnPrinter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.edn {
            edn::Value::Nil => write!(f, "nil"),
            edn::Value::Boolean(b) => write!(f, "{}", &b),
            edn::Value::String(s) => write!(f, "{}", &s),
            edn::Value::Char(c) => write!(f, "{}", &c),
            edn::Value::Symbol(s) => write!(f, "{}", &s),
            edn::Value::Keyword(k) => write!(f, "{}", &k),
            edn::Value::Integer(i) => write!(f, "{}", &i),
            edn::Value::Float(flt) => write!(f, "{}", &flt),
            edn::Value::List(values) => {
                write!(f, "(")?;
                values
                    .iter()
                    .map(EdnPrinter::from)
                    .try_fold(true, |is_first, value| {
                        if !is_first {
                            write!(f, ",")?;
                        }

                        write!(f, "{}", value)?;
                        Ok(false)
                    })?;

                write!(f, ")")
            }
            edn::Value::Vector(values) => {
                write!(f, "[")?;
                values
                    .iter()
                    .map(EdnPrinter::from)
                    .try_fold(true, |is_first, value| {
                        if !is_first {
                            write!(f, ",")?;
                        }

                        write!(f, "{}", value)?;
                        Ok(false)
                    })?;
                write!(f, "]")
            }
            edn::Value::Map(m) => {
                write!(f, "{}", "{")?;
                m.iter()
                    .map(|(k, v)| (EdnPrinter::from(k), EdnPrinter::from(v)))
                    .try_fold(true, |is_first, (k, v)| {
                        if !is_first {
                            write!(f, ",")?;
                        }

                        write!(f, "{} {}", k, v)?;
                        Ok(false)
                    })?;
                write!(f, "{}", "}")
            }
            edn::Value::Set(values) => {
                write!(f, "{}", "#{")?;
                values
                    .iter()
                    .map(|v| EdnPrinter::from(v))
                    .try_fold(true, |is_first, v| {
                        if !is_first {
                            write!(f, ",")?;
                        }

                        write!(f, "{}", v)?;
                        Ok(false)
                    })?;
                write!(f, "{}", "}")
            }
            // edn::Value::Tagged(tag, value) => {
            //     write!(f, "#{}", tag)?;
            //     write!(f, " {}", EdnPrinter::from(value))
            // }
            _ => write!(f, "Nope!"),
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(io::stdout());

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let mut parser = Parser::new(&line);
        if let Some(result) = parser.read() {
            let edn = result.unwrap();
            let printer = EdnPrinter::new(&edn);
            println!("{}", printer);
        }
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1)
        }
    }
}
