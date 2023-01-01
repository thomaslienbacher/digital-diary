use std::path::PathBuf;

use ansi_term::Color::*;
use chrono::{DateTime, Local};
use dirs::home_dir;
use rusqlite::{Connection, OpenFlags};
use rusqlite::params;
use sha2::{Digest, Sha256};
use whoami::username;

#[derive(Clone, Debug)]
pub struct Entry {
    id: i64,
    hash: Vec<u8>,
    date: DateTime<Local>,
    keywords: Vec<String>,
    title: String,
    content: String,
    hidden: bool,
}

pub struct Diary {
    connection: Connection,
}

impl Diary {
    fn get_database_url(expect_existence: bool) -> PathBuf {
        if let Ok(a) = std::env::var("DIDI_URL") {
            let p = PathBuf::from(a);

            if !p.exists() && expect_existence {
                panic!("Error: database specified in DIDI_URL doesn't exist maybe use `didi create`")
            }

            p
        } else {
            match home_dir() {
                Some(mut d) => {
                    d.push("digital_diary.sqlite");

                    if !d.as_path().exists() && expect_existence {
                        panic!("Error: no database file found. Specify DIDI_URL or use `didi create`")
                    }

                    d
                }
                None => panic!("Error: couldn't retrieve home directory")
            }
        }
    }

    pub fn open() -> Self {
        let url = Self::get_database_url(true);
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX | OpenFlags::SQLITE_OPEN_URI;
        let connection = match Connection::open_with_flags(&url, flags) {
            Ok(c) => c,
            Err(e) => panic!("Error: couldn't open database connection: {:?}", e)
        };

        println!("Welcome {} at '{}'!\n", Cyan.paint(username()),
                 Cyan.paint(url.as_path().to_str().unwrap()));

        Diary { connection }
    }

    pub fn create() -> PathBuf {
        let url = Self::get_database_url(false);
        let connection = match Connection::open_with_flags(&url, OpenFlags::default()) {
            Ok(c) => c,
            Err(e) => panic!("Error: couldn't open database connection: {:?}", e)
        };

        if let Err(e) = connection.execute(
            r#"create table entries
                (
                    id       INTEGER not null,
                    hash     BLOB    not null,
                    date     TEXT    not null,
                    keywords TEXT    not null,
                    title    TEXT    not null,
                    content  TEXT    not null,
                    hidden   INTEGER not null,
                    primary key (id autoincrement),
                    unique (id)
                );"#, []) {
            panic!("Error: couldn't create database tables: {:?}", e)
        }

        url
    }

    /// Adds an entry to the database
    ///
    /// `keywords` have to be lowercase
    pub fn add(&mut self, mut keywords: Vec<String>, title: String, content: String) {
        keywords.sort();
        keywords.dedup();
        let keywords_str = keywords.join(";");
        let now = Local::now().to_rfc3339();

        let hash: Vec<u8> = {
            let mut hasher = Sha256::new();
            hasher.update(&keywords_str);
            hasher.update(&title);
            hasher.update(&content);
            hasher.update(&now);
            hasher.finalize().to_vec()
        };

        match self.connection.execute(
            r#"
            INSERT INTO entries (hash, date, keywords, title, content, hidden) VALUES
            (?1, ?2, ?3, ?4, ?5, false)
            "#, params![hash, now, keywords_str, title, content]) {
            Err(e) => panic!("Error: couldn't insert entry: {:?}", e),
            Ok(_) => println!("Added {}!", Cyan.paint(title))
        }
    }

    /// Retrieves all entries from the database
    fn get_entries(&mut self) -> Vec<Entry> {
        let mut stmt = match self.connection.prepare("SELECT * FROM entries") {
            Ok(o) => o,
            Err(e) => panic!("Error: can't build sql statement: {:?}", e)
        };

        stmt.query_map(params![], |row| {
            let id = row.get(0)?;
            let hash = row.get(1)?;
            let date = row.get(2)?;
            let keywords: Vec<String> = {
                let k: String = row.get(3)?;
                k.split(';').map(|s| s.to_string()).collect()
            };
            let title = row.get(4)?;
            let content = row.get(5)?;
            let hidden = row.get(6)?;

            Ok(Entry {
                id,
                hash,
                date,
                keywords,
                title,
                content,
                hidden,
            })
        }).unwrap().map(|r| r.unwrap()).collect()
    }

    /// Prints the given entries, which and what gets printed can be customised using the
    /// parameters. If a parameter is `true` it will get printed. Hidden entries
    /// will get printed if `hidden` is `true`.
    fn print_entries(entries: Vec<Entry>, date: bool, id: bool, hash: bool, keywords: bool,
                     content: bool, hidden: bool) {
        let mut counter = 0;
        let mut iter = entries.into_iter().filter(|a| !a.hidden || hidden);

        loop {
            match iter.next() {
                Some(e) => {
                    println!("{:-<1$}\n", "", termsize::get().unwrap().cols as usize);
                    counter += 1;

                    let title = format!("{}", Cyan.underline().paint(e.title));
                    print!("{:<40}", title);

                    if date {
                        print!("{} ", Cyan.paint(e.date.to_rfc2822().to_string()));
                    }

                    if id {
                        let id = format!("{}", Cyan.paint(format!("[{}]", e.id)));
                        print!("{:<20}", id);
                    }

                    if hash {
                        let hash = format!("[{}]", hex::encode(&e.hash));
                        print!("{:<30}", Cyan.paint(hash))
                    }

                    println!();

                    if keywords {
                        print!("Keywords: ");

                        let last = e.keywords.last().unwrap().clone();
                        for k in e.keywords {
                            print!("{}", Cyan.paint(&k));
                            if k != last {
                                print!(", ")
                            }
                        }

                        println!();
                    }

                    if content {
                        println!("{}", e.content)
                    }

                    println!();
                }
                None => {
                    if counter > 0 {
                        println!("{:-<1$}", "", termsize::get().unwrap().cols as usize);
                    }
                    break;
                }
            }
        }

        if counter == 1 {
            println!("Found {} entry.", Cyan.paint(format!("{}", counter)));
        } else {
            println!("Found {} entries.", Cyan.paint(format!("{}", counter)));
        }
    }

    /// Prints all entries, which and what gets printed can be customised using the
    /// parameters.
    pub fn list_all(&mut self, date: bool, id: bool, hash: bool, keywords: bool, content: bool,
                    hidden: bool) {
        let entries = self.get_entries();
        Self::print_entries(entries, date, id, hash, keywords, content, hidden);
    }

    /// Searches through all entries and prints the one that match the search terms,
    /// which and what gets printed can be customised using the parameters.
    /// `searchfor` contains the words to search for, from every entry the keywords and the title will
    /// be searched. `searchfor` has to be lowercase.
    pub fn search(&mut self, searchfor: Vec<String>, date: bool, id: bool, hash: bool, keywords: bool,
                  content: bool, hidden: bool) {
        let entries = self.get_entries();
        let mut found = Vec::new();

        for e in &entries {
            for s in &searchfor {
                let mut jumpout = false;

                if e.title.to_lowercase().contains(s) {
                    found.push(e.clone());
                    break;
                }

                for k in &e.keywords {
                    if k.contains(s) {
                        found.push(e.clone());
                        jumpout = true;
                        break;
                    }
                }

                if jumpout {
                    break;
                }
            }
        }

        Self::print_entries(found, date, id, hash, keywords, content, hidden);
    }

    /// Hides or unhides the entries given by `ids`.
    /// The `set` parameter specifies if the entry should be hidden or not.
    pub fn hide(&mut self, ids: Vec<i64>, set: bool) {
        let mut counter = 0;

        for i in ids {
            match self.connection.execute(
                r#"
                UPDATE entries SET hidden = ?1 WHERE id = ?2
                "#, params![set, i]) {
                Err(e) => panic!("Error: couldn't update entry: {:?}", e),
                Ok(n) => counter += n
            }
        }

        if counter == 1 {
            println!("Changed {} entry.", Cyan.paint(format!("{}", counter)));
        } else {
            println!("Changed {} entries.", Cyan.paint(format!("{}", counter)));
        }
    }
}
