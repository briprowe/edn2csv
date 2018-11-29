#![feature(transpose_result)]

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
        EdnPrinter::new(edn)
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
            edn::Value::Tagged(tag, value) => {
                write!(f, "#{}", tag)?;
                write!(f, " {}", EdnPrinter::from(value.as_ref()))
            }
        }
    }
}

#[derive(Debug)]
struct ParseError {
    linenum: usize,
    cause: edn::parser::Error,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({}, {}): {} ",
            self.linenum, self.cause.lo, self.cause.hi, self.cause.message
        )
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "Error while reading EDN"
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    use std::collections::BTreeSet;

    let mut records = vec![];
    let mut columns: BTreeSet<String> = BTreeSet::new();

    let stdin = io::stdin();
    for (idx, line) in stdin.lock().lines().enumerate() {
        let line = line?;
        let mut parser = Parser::new(&line);

        if let Some(edn) = parser.read().transpose().or_else(|e| {
            Err(ParseError {
                linenum: idx,
                cause: e,
            })
        })? {
            match edn {
                edn::Value::Map(m) => {
                    let keys: Vec<String> = m
                        .keys()
                        .filter_map(|key| match key {
                            edn::Value::Keyword(k) => Some(k.clone()),
                            _ => {
                                eprintln!("Skipping non keyword key: {}", EdnPrinter::new(&key));
                                None
                            }
                        })
                        .collect();

                    columns.extend(keys);
                    records.push(m);
                }
                _ => eprintln!("Skipping non map on line {}", idx),
            }
        }
    }

    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(io::stdout());

    for c in &columns {
        writer.write_field(c)?;
    }
    writer.write_record(None::<&[u8]>)?;

    let columns: Vec<edn::Value> = columns
        .iter()
        .map(|k| edn::Value::Keyword(k.to_string()))
        .collect();

    for r in records {
        for c in &columns {
            if let Some(field) = r
                .get(c)
                .and_then(|f| Some(format!("{}", EdnPrinter::new(f))))
            {
                writer.write_field(field.as_bytes())?;
            } else {
                writer.write_field("")?;
            }
        }
        writer.write_record(None::<&[u8]>)?;
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
