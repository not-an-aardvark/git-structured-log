extern crate chrono;
extern crate git2;
extern crate serde_json;

use chrono::{NaiveDateTime, DateTime, FixedOffset};
use git2::{Object, Oid, Repository, Time};
use serde_json::{map::Map, value::Value};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => match print_commits(&args[1], &args[2]) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("fatal: {}", err);
                process::exit(1)
            }
        }
        _ => {
            eprintln!("{}", "Usage: git_structured_log <exclusive start of range>..<inclusive end of range> <comma-separated list of format flags>");
            process::exit(1)
        }
    }
}

fn print_commits(revision_range: &str, formats_input: &str) -> Result<(), Box<Error>> {
    let repository: &mut Repository = &mut Repository::open(".")?;

    let formats = formats_input.split(',').collect::<Vec<&str>>();
    let mut reference_map: HashMap<Oid, Vec<String>> = HashMap::new();

    if formats.contains(&&"D") {
        for reference_result in repository.references()? {
            let reference = reference_result?;
            let ref_shorthand = match reference.shorthand() {
                Some(shorthand) => shorthand,
                None => continue
            };
            let ref_target = reference.peel_to_commit()?.id();
            reference_map.entry(ref_target).or_insert_with(Vec::new).push(ref_shorthand.to_string());
        }
    }

    let mut revwalk = repository.revwalk()?;
    revwalk.push_range(revision_range)?;

    for oid in revwalk {
        let commit = repository.find_commit(oid?)?;
        let mut map = Map::new();
        for format in &formats {
            map.insert(format.to_string(), match *format {
                "H" => Value::String(oid_to_hex_string(commit.id())),
                "h" => Value::String(object_to_hex_string(commit.as_object())?),
                "T" => Value::String(oid_to_hex_string(commit.tree_id())),
                "t" => Value::String(object_to_hex_string(commit.tree()?.as_object())?),
                "P" => commit.parent_ids().map(oid_to_hex_string).map(Value::String).collect::<Value>(),
                "p" => commit.parents()
                    .map(|parent| Ok(Value::String(object_to_hex_string(parent.as_object())?)))
                    .collect::<Result<Value, Box<Error>>>()?,
                "an" => Value::String(commit.author().name().ok_or("Author name contains invalid UTF8")?.to_string()),
                "ae" => Value::String(commit.author().email().ok_or("Author email contains invalid UTF8")?.to_string()),
                "aN" | "aE" => invalid_format(format, "Mailmaps not currently supported, consider using `an`/`ae` instead of `aN`/`aE`")?,
                "at" => Value::Number(commit.author().when().seconds().into()),
                "aI" => Value::String(git_time_to_iso8601(commit.author().when())),
                "ad" | "aD" | "ar" | "ai" => invalid_format(format, "Formatted dates not supported, use `aI` and format the date yourself")?,
                "ct" => Value::Number(commit.time().seconds().into()),
                "cI" => Value::String(git_time_to_iso8601(commit.time())),
                "cd" | "cD" | "cr" | "ci" => invalid_format(format, "Formatted dates not supported, use `cI` and format the date yourself")?,
                "d" => invalid_format(format, "Formatted ref names not supported, use `D` and format the names yourself")?,
                "D" => reference_map
                    .remove(&commit.id())
                    .unwrap_or_else(Vec::new)
                    .into_iter()
                    .map(Value::String)
                    .collect::<Value>(),
                "s" => Value::String(commit.summary().ok_or("Commit header contains invalid UTF8")?.to_string()),
                "b" => invalid_format(format, "Body not supported, use `B` and extract the body yourself")?,
                "B" => Value::String(commit.message().ok_or("Commit message contains invalid UTF8")?.to_string()),
                "N" => invalid_format(format, "Notes not currently supported")?,
                "GG" | "G?" | "GS" | "GK" => invalid_format(format, "Signatures not currently supported")?,
                _ => invalid_format(format, "Not found")?
            });
        }
        println!("{}", Value::Object(map));
    }

    Ok(())
}

fn oid_to_hex_string(oid: Oid) -> String {
    oid.as_bytes().into_iter().map(|byte| format!("{:02x}", byte)).collect::<String>()
}

fn object_to_hex_string(object: &Object) -> Result<String, Box<Error>> {
    match object.short_id()?.as_str() {
        Some(shorthash) => Ok(shorthash.to_string()),
        None => Err("libgit returned a bad shorthash")?
    }
}

fn git_time_to_iso8601(time: Time) -> String {
    let time_without_zone = NaiveDateTime::from_timestamp(time.seconds(), 0);
    let time_with_zone = DateTime::<FixedOffset>::from_utc(time_without_zone, FixedOffset::east(time.offset_minutes() * 60));
    time_with_zone.to_rfc3339()
}

fn invalid_format(format: &str, reason: &str) -> Result<Value, Box<Error>> {
    Err(format!("Invalid format `{}`: {}", format, reason))?
}
