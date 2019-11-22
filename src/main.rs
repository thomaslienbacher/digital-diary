mod diary;

use crate::diary::*;
use clap::{Arg, App};
use ansi_term::Color::*;
use text_io::*;
use std::io::{Write, stdout, stdin};

fn main() {
    #[cfg(windows)] {
        if let Err(e) = ansi_term::enable_ansi_support() {
            panic!("Error: couldn't enable ansi escape codes: GetLastError = {}", e);
        }
    }

    let display_args = [
        Arg::with_name("nocontent")
            .short("n")
            .long("nocontent")
            .help("Don't show content"),
        Arg::with_name("id")
            .short("i")
            .long("id")
            .help("Show id of entry"),
        Arg::with_name("hash")
            .short("h")
            .long("hash")
            .help("Show hash of entry"),
        Arg::with_name("keywords")
            .short("k")
            .long("keywords")
            .help("Show keywords of entry"),
        Arg::with_name("nodate")
            .short("d")
            .long("nodate")
            .help("Don't show date"),
        Arg::with_name("hidden")
            .short("a")
            .long("hidden")
            .help("Show hidden entries")];

    let tohide = Arg::with_name("tohide")
        .required(true)
        .multiple(true)
        .validator(|a| {
            match a.parse::<u64>() {
                Err(_) => return Err(String::from("argument only accepts positive numbers (u64)")),
                _ => return Ok(())
            }
        })
        .help("Ids of the entries to update");

    let matches = App::new("Digital Diary")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Thomas Lienbacher <lienbacher.tom@gmail.com>")
        .about("A small CLI diary used to document your life.")
        .subcommand(
            App::new("create")
                .about("Creates the database")
        )
        .subcommand(
            App::new("add")
                .about("Adds an entry")
        )
        .subcommand(
            App::new("list")
                .about("Lists all entries")
                .args(&display_args))
        .subcommand(
            App::new("search")
                .about("Searches for entries")
                .arg(Arg::with_name("searchfor")
                    .required(true)
                    .multiple(true)
                    .help("Keywords to search for"))
                .args(&display_args)
        )
        .subcommand(App::new("hide")
            .about("Hide one or more entries")
            .arg(&tohide)
        )
        .subcommand(App::new("unhide")
            .about("Unhide one or more entries")
            .arg(&tohide)
        )
        .get_matches();

    match matches.subcommand() {
        ("create", Some(_)) => {
            let url = Diary::create();
            println!("Created database at '{}'!", Cyan.paint(url.as_path().to_str().unwrap()))
        }
        ("add", Some(_)) => {
            let mut diary = Diary::open();

            print!("{}", Cyan.paint("Title: "));
            stdout().flush().unwrap();
            let title: String = read!("{}\n");

            print!("{}", Cyan.paint("Content: "));
            stdout().flush().unwrap();
            let mut content = String::new();
            while let Ok(_) = stdin().read_line(&mut content) {
                if content.ends_with("\r\n\r\n") || content.ends_with("\n\n") {
                    break;
                }
            }

            print!("{}", Cyan.paint("Keywords: "));
            stdout().flush().unwrap();
            let keywords = {
                let raw: String = read!("{}\n");
                raw.trim().split_whitespace().map(|s| s.trim().to_lowercase()).collect()
            };

            diary.add(keywords, title.trim().into(), content.trim().into());
        }
        ("list", Some(matches)) => {
            let mut diary = Diary::open();

            let date = !matches.is_present("nodate");
            let id = matches.is_present("id");
            let hash = matches.is_present("hash");
            let keywords = matches.is_present("keywords");
            let content = !matches.is_present("nocontent");
            let hidden = matches.is_present("hidden");

            diary.list_all(date, id, hash, keywords, content, hidden);
        }
        ("search", Some(matches)) => {
            let mut diary = Diary::open();

            let keywords: Vec<String> = matches.values_of("searchfor").unwrap()
                .map(|s| s.to_lowercase()).collect();

            let date = !matches.is_present("nodate");
            let id = matches.is_present("id");
            let hash = matches.is_present("hash");
            let showkeywords = matches.is_present("keywords");
            let content = !matches.is_present("nocontent");
            let hidden = matches.is_present("hidden");

            diary.search(keywords, date, id, hash, showkeywords, content, hidden);
        }
        ("hide", Some(matches)) => {
            let mut diary = Diary::open();

            let mut ids: Vec<i64> = matches.values_of("tohide").unwrap()
                .map(|s| s.parse().unwrap()).collect();
            ids.sort();
            ids.dedup();

            diary.hide(ids, true);
        }
        ("unhide", Some(matches)) => {
            let mut diary = Diary::open();

            let mut ids: Vec<i64> = matches.values_of("tohide").unwrap()
                .map(|s| s.parse().unwrap()).collect();
            ids.sort();
            ids.dedup();

            diary.hide(ids, false);
        }
        ("", _) => {
            println!("No subcommand given. Use flag --help for more information.");
        }
        _ => unreachable!()
    }

    stdout().flush().unwrap();
}
