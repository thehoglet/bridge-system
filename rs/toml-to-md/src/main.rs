use data_model::Continuation;
use data_model::Open;
use glob::glob;
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::{error::Error, process};
use text_colorizer::*;

#[derive(Debug)]
struct Arguments {
    source_toml: String,
    target_directory: String,
}

fn print_usage() {
    eprintln!(
        "{} - format bidding agreements encoded in TOML as Markdown",
        "toml-to-md".green()
    );
    eprintln!(
        "Usage: toml-to-md <source TOML file or directory> <target directory>"
    );
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
        source_toml: args[0].to_owned(),
        target_directory: args[1].to_owned(),
    }
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        process::exit(1);
    }
}

fn encode_continuation_rec<'a>(
    sequence: &mut Vec<&'a String>,
    bid: &'a String,
    meaning: &'a String,
    maybe_notes: &Option<Vec<String>>,
    maybe_rebid: &Option<String>,
    pass: &'a BTreeMap<String, Continuation>,
    markdown: &mut Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    writeln!(markdown)?;
    writeln!(
        markdown,
        "## {}&nbsp;{}",
        sequence.iter().join("&nbsp;"),
        bid
    )?;

    writeln!(markdown)?;
    writeln!(markdown, "{}", meaning)?;

    if let Some(notes) = maybe_notes {
        writeln!(markdown)?;
        writeln!(markdown, "{}", notes.join("; "))?;
    }

    if let Some(rebid) = maybe_rebid {
        writeln!(markdown)?;
        writeln!(markdown, "{}", rebid)?;
    }

    sequence.push(bid);

    encode_continuations(sequence, pass, markdown)?;

    for bid in pass.keys().sorted_by(cmp_bids) {
        let continuation = &pass[bid];
        if let Some(needs_encoding) = &continuation.pass {
            encode_continuation_rec(
                sequence,
                bid,
                &continuation.meaning,
                &continuation.notes,
                &continuation.rebid,
                needs_encoding,
                markdown,
            )?;
        }
    }

    sequence.pop();

    Ok(())
}

fn format_link_to_anchor(sequence: &Vec<&String>) -> String {
    let text = sequence.iter().join("&nbsp;");
    let link = sequence
        .iter()
        .map(|&s| s.to_lowercase().replace("/", ""))
        .join("-");
    format!("[{}](#{})", text, link)
}

fn cmp_bids(lhs: &&String, rhs: &&String) -> Ordering {
    let Ok(lhs_level) = lhs[0..1].parse::<u8>() else {
        return Ordering::Less;
    };
    let Ok(rhs_level) = rhs[0..1].parse::<u8>() else {
        return Ordering::Greater;
    };

    const NT_SUBS: &str = "ZNT";

    if lhs_level == rhs_level {
        let mut lhs_suit = &lhs[1..];
        let mut rhs_suit = &rhs[1..];
        if lhs_suit == "NT" {
            lhs_suit = NT_SUBS;
        }
        if rhs_suit == "NT" {
            rhs_suit = NT_SUBS;
        }
        return Ord::cmp(&lhs_suit, &rhs_suit);
    }

    Ord::cmp(&lhs_level, &rhs_level)
}

fn encode_continuations<'a>(
    sequence: &mut Vec<&'a String>,
    pass: &'a BTreeMap<String, Continuation>,
    markdown: &mut Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    let column1_heading = if sequence.is_empty() {
        "Response"
    } else {
        "Rebid"
    };

    writeln!(markdown)?;
    writeln!(markdown, "| {} | Meaning |", column1_heading)?;
    writeln!(markdown, "|---|---|")?;

    for bid in pass.keys().sorted_by(cmp_bids) {
        sequence.push(bid);
        let continuation = &pass[bid];
        let column1 = if continuation.pass.is_some() {
            format_link_to_anchor(sequence)
        } else {
            sequence.iter().join("&nbsp;")
        };

        if continuation.notes.is_none() && continuation.rebid.is_none() {
            writeln!(markdown, "| {} | {} |", column1, continuation.meaning)?;
        } else {
            let mut additional: String = "".into();
            if let Some(notes) = &continuation.notes {
                additional.push_str(&notes.join("; "));
            }
            if let Some(rebid) = &continuation.rebid {
                if !additional.is_empty() {
                    additional.push_str("; ");
                }
                additional.push_str(&rebid);
            }
            writeln!(
                markdown,
                "| {} | {} ({}) |",
                column1, continuation.meaning, additional
            )?;
        }

        sequence.pop();
    }

    Ok(())
}

fn encode_open(
    open: &Open,
    markdown: &mut Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    writeln!(markdown, "# {}", open.open)?;

    writeln!(markdown)?;
    writeln!(markdown, "{}", open.meaning)?;

    if let Some(notes) = &open.notes {
        writeln!(markdown)?;
        writeln!(markdown, "{}", notes.join("; "))?;
    }

    if let Some(fourth) = &open.fourth {
        writeln!(markdown)?;
        writeln!(markdown, "{}", fourth)?;
    }

    if let Some(pass) = &open.pass {
        let mut sequence = vec![&open.open];

        encode_continuations(&mut sequence, pass, markdown)?;

        for bid in pass.keys().sorted_by(cmp_bids) {
            let continuation = &pass[bid];
            if let Some(needs_encoding) = &continuation.pass {
                encode_continuation_rec(
                    &mut sequence,
                    bid,
                    &continuation.meaning,
                    &continuation.notes,
                    &continuation.rebid,
                    needs_encoding,
                    markdown,
                )?;
            }
        }
    }

    writeln!(markdown)?;
    write!(markdown, "[Home](../index.md)")?;

    writeln!(markdown)?;

    Ok(())
}

fn process_toml(
    source_path: &PathBuf,
    target_path: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&source_path)?;
    let open = toml::from_str(&contents)?;

    let source_metadata = fs::metadata(source_path)?;
    let mut markdown =
        Vec::<u8>::with_capacity(source_metadata.len() as usize * 2);

    encode_open(&open, &mut markdown)?;

    let mut target_file = target_path.clone();
    let mut file_name: String = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .expect(&format!(
            "Failed to extract file name from {:?}",
            source_path
        ))
        .into();
    file_name = file_name
        .replace(" ", "_")
        .replace("(", "")
        .replace(")", "");
    target_file.push(file_name);

    if !target_file.is_dir() {
        fs::create_dir_all(&target_file)?;
    }

    target_file.push("index.md");

    fs::write(target_file, &markdown)?;

    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = parse_args();

    let source_path = PathBuf::from(&args.source_toml);

    if !source_path.exists() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source TOML not found: {}", &args.source_toml),
        )));
    }

    let target_path = PathBuf::from(&args.target_directory);

    if !target_path.is_dir() {
        fs::create_dir_all(&target_path)?;
    }

    if source_path.is_file() {
        process_toml(&source_path, &target_path)?;
        return Ok(());
    }

    let mut path_to_glob = source_path.clone();
    path_to_glob.push("*.toml");
    let pattern = path_to_glob.to_str().expect("Failed to form TOML glob");
    let paths = glob(pattern).expect("Failed to read TOML source directory");

    for entry in paths {
        match entry {
            Ok(source_file) => {
                process_toml(&source_file, &target_path)?;
            }
            Err(e) => println!("{:?}", e),
        }
    }

    Ok(())
}
