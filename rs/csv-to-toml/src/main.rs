use data_model::Continuation;
use data_model::Open;
use glob::glob;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::ops::Range;
use std::path::Path;
use std::path::PathBuf;
use std::{error::Error, process};
use text_colorizer::*;

static PATTERN_BID_MEANING: &str = r"^(?P<bid>(?:\d(?:[SHDC]|NT)|(?:\(?[Pp]ass\)?)))(?P<bids>/[^-–]*)?\*?(?:(?:\s*[-–]\s*(?P<meaning>.*))|$)";
lazy_static! {
    static ref RE_BID_MEANING: Regex = Regex::new(PATTERN_BID_MEANING).unwrap();
}

#[derive(Debug)]
struct Arguments {
    source_csv: String,
    target_directory: String,
}

fn print_usage() {
    eprintln!(
        "{} - encode bidding agreements as TOML",
        "csv-to-toml".green()
    );
    eprintln!("Usage: csv-to-toml <source csv file> <target directory>");
}

fn parse_args() -> Arguments {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 2 {
        print_usage();
        eprintln!(
            "{} wrong number of arguments: expected 2, got {}.",
            "Error:".red().bold(),
            args.len()
        );
        std::process::exit(1);
    }
    Arguments {
        source_csv: args[0].to_owned(),
        target_directory: args[1].to_owned(),
    }
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}

fn process_continuations_rec(
    pass: &mut BTreeMap<String, Continuation>,
    field_values: &Vec<Vec<String>>,
    column: usize,
    rows: Range<usize>,
) -> Result<Option<String>, Box<dyn Error>>
where
    Range<usize>: Iterator<Item = usize>,
{
    let n_columns = field_values[0].len();
    let right_open_bound = rows.end;
    let mut continuation_bid: String = "".into();
    let mut continuation = Continuation::default();
    let mut collect_text = true;
    let mut start_from_row: usize = 0;
    let mut maybe_rebid: Option<String> = None;

    fn recurse(
        continuation: &mut Continuation,
        field_values: &Vec<Vec<String>>,
        column: usize,
        rows: Range<usize>,
    ) -> Result<(), Box<dyn Error>> {
        let mut pass_rec = BTreeMap::<String, Continuation>::new();

        let maybe_rebid = process_continuations_rec(
            &mut pass_rec,
            field_values,
            column,
            rows,
        )?;

        if let Some(rebid) = maybe_rebid {
            if continuation.meaning.is_empty() {
                continuation.meaning = rebid;
            } else {
                continuation.rebid = Some(rebid);
            }
        }

        if !pass_rec.is_empty() {
            continuation.pass = Some(pass_rec);
        }

        Ok(())
    }

    for row in rows {
        let value = &field_values[row][column];
        if value.is_empty() {
            continue;
        }

        let maybe_match = if value.contains(';') {
            None
        } else {
            RE_BID_MEANING.captures(value)
        };

        match maybe_match {
            Some(captures) => {
                let bid = captures.name("bid").unwrap().as_str().into();
                if !continuation_bid.is_empty() {
                    if continuation_bid == bid {
                        collect_text = false;
                    } else {
                        if n_columns > column + 1 {
                            recurse(
                                &mut continuation,
                                field_values,
                                column + 1,
                                start_from_row..row,
                            )?;
                        }

                        if continuation.meaning.is_empty() {
                            return Err(Box::new(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!(
                                    "Found undefined continuation: {}",
                                    continuation_bid
                                ),
                            )));
                        }

                        pass.insert(continuation_bid, continuation);
                        continuation_bid = "".into();
                        continuation = Continuation::default();
                    }
                }
                if continuation_bid.is_empty() {
                    continuation_bid = bid;
                    start_from_row = row;
                    collect_text = true;
                    if let Some(bids) = captures.name("bids") {
                        continuation_bid.push_str(bids.into());
                    }
                    if let Some(meaning) = captures.name("meaning") {
                        continuation.meaning = meaning.as_str().into();
                    }
                }
            }
            None => {
                if continuation_bid.is_empty() {
                    if let Some(rebid) = &mut maybe_rebid {
                        rebid.push_str("; ");
                        rebid.push_str(value);
                    } else {
                        maybe_rebid = Some(value.into());
                    }
                }
                if collect_text {
                    if continuation.meaning.is_empty() {
                        continuation.meaning = value.into();
                    } else {
                        let notes =
                            continuation.notes.get_or_insert_with(Vec::new);
                        notes.push(value.into());
                    }
                }
            }
        }
    }

    if !continuation_bid.is_empty() {
        if n_columns > column + 1 {
            recurse(
                &mut continuation,
                field_values,
                column + 1,
                start_from_row..right_open_bound,
            )?;
        }

        if continuation.meaning.is_empty() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Found undefined continuation: {}", continuation_bid),
            )));
        }

        pass.insert(continuation_bid, continuation);
    }

    Ok(maybe_rebid)
}

pub fn create_open(
    field_values: &Vec<Vec<String>>,
) -> Result<Open, Box<dyn Error>> {
    let mut open = Open::default();
    let mut collect_text = true;
    let mut start_from_row: usize = 0;
    let mut end_before_row: usize = usize::MAX;

    let fourth_prefixes = ["fourth", "4th"];

    for (pos, row) in field_values.iter().enumerate() {
        let opener = &row[0];
        if opener.is_empty() {
            continue;
        }

        let maybe_match = if opener.contains(';') {
            None
        } else {
            RE_BID_MEANING.captures(opener)
        };

        match maybe_match {
            Some(captures) => {
                let bid = captures.name("bid").unwrap().as_str().into();
                if !open.open.is_empty() {
                    if open.open == bid {
                        collect_text = false;
                    } else {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Found additional opening bid {}", bid),
                        )));
                    }
                } else {
                    open.open = bid;
                    start_from_row = pos;

                    if let Some(bids) = captures.name("bids") {
                        open.open.push_str(bids.into());
                    }

                    if let Some(meaning) = captures.name("meaning") {
                        open.meaning = meaning.as_str().into();
                    }
                }
            }
            None => {
                if fourth_prefixes
                    .iter()
                    .any(|&s| opener.to_lowercase().starts_with(s))
                {
                    if open.meaning.is_empty() {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Found 4th position definition for undefined open",
                        )));
                    }
                    if open.fourth.is_some() {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Found additional 4th position definition for {}", open.meaning),
                        )));
                    }
                    open.fourth = Some(opener.into());
                    end_before_row = pos;
                } else if collect_text {
                    if open.meaning.is_empty() {
                        open.meaning = opener.into();
                    } else {
                        let notes = open.notes.get_or_insert_with(Vec::new);
                        notes.push(opener.into());
                    }
                }
            }
        }
    }

    if open.open.is_empty() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "Failed to find opening bid",
        )));
    } else if field_values[0].len() > 1 {
        let mut pass = BTreeMap::<String, Continuation>::new();

        if end_before_row == usize::MAX {
            end_before_row = field_values.len();
        }

        let maybe_rebid = process_continuations_rec(
            &mut pass,
            field_values,
            1,
            start_from_row..end_before_row,
        )?;

        if let Some(rebid) = maybe_rebid {
            let notes = open.notes.get_or_insert_with(Vec::new);
            notes.push(rebid);
        }

        if !pass.is_empty() {
            open.pass = Some(pass);
        }
    }

    Ok(open)
}

fn process_csv(
    source_path: &Path,
    target_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(source_path)?;

    let mut field_values: Vec<Vec<String>> = Vec::new();

    for result in reader.records() {
        let record = result?;
        let row: Vec<String> = record
            .iter()
            .map(|field| field.trim().to_string())
            .collect();

        if row.iter().all(|s| s.is_empty()) {
            continue;
        }

        if row[0].to_lowercase() == "opener"
            && row[1].to_lowercase() == "responder"
        {
            continue;
        }

        field_values.push(row);
    }

    if field_values.is_empty() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            "CSV file contains no processable data",
        )));
    }

    let open = create_open(&field_values)?;

    let file_name = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| {
            panic!("Failed to extract file name from {:?}", source_path)
        });

    let mut target_file = target_path.to_path_buf();
    target_file.push(file_name);
    target_file.set_extension("toml");

    let content = toml::to_string(&open)?;
    fs::write(&target_file, content)?;

    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = parse_args();

    let source_path = PathBuf::from(&args.source_csv);

    if !source_path.exists() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source CSV not found: {}", &args.source_csv),
        )));
    }

    let target_path = PathBuf::from(&args.target_directory);

    if !target_path.is_dir() {
        fs::create_dir_all(&target_path)?;
    }

    if source_path.is_file() {
        process_csv(&source_path, &target_path)?;
        return Ok(());
    }

    let mut path_to_glob = source_path.clone();
    path_to_glob.push("*.csv");
    let pattern = path_to_glob.to_str().expect("Failed to form CSV glob");
    let paths = glob(pattern).expect("Failed to read CSV source directory");

    for entry in paths {
        match entry {
            Ok(source_file) => {
                process_csv(&source_file, &target_path)?;
            }
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}
