use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

use walkdir::{DirEntry, WalkDir};

const START: &str = "@MANSTART";
const END: &str = "@MANEND";

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn main() {
    let mut args = env::args().skip(1);
    let source = args.next().unwrap_or(".".to_string());
    let output = args.next().unwrap_or("man".to_string());

    let walker = WalkDir::new(&source).follow_links(true).into_iter();
    for entry in walker
        .filter_entry(|e| !is_hidden(e))
        .map(|x| x.expect("failed to read entry"))
        .filter(|x| x.file_type().is_file())
    {
        match File::open(entry.path()) {
            Ok(mut file) => {
                let mut string = String::new();
                match file.read_to_string(&mut string) {
                    Ok(_) => {
                        for i in string.split(START).skip(1) {
                            let start_delimiter = i.find('{').unwrap_or_else(|| {
                                panic!("{}: No opened '{{' for MANSTART", entry.path().display())
                            });
                            let end_delimiter = i.find('}').unwrap_or_else(|| {
                                panic!("{}: Unclosed '{{' for MANSTART", entry.path().display())
                            });
                            let name = &i[start_delimiter + 1..end_delimiter];
                            assert!(
                                name.lines().count() == 1,
                                "{}: malformed manpage name",
                                entry.path().display()
                            );

                            let man_page = &i[end_delimiter + 1
                                ..i.find(END).unwrap_or_else(|| {
                                    panic!(
                                        "{}: Unclosed @MANSTART (use @MANEND)",
                                        entry.path().display()
                                    )
                                }) + END.len()]
                                .trim();

                            let mut string =
                                String::with_capacity(man_page.len() + man_page.len() / 3);

                            for i in man_page.lines().skip(1) {
                                if !i.contains(END) {
                                    string.push_str(
                                        i.trim_start_matches('\\')
                                            .trim_start_matches("// ")
                                            .trim_start_matches("//! ")
                                            .trim_start_matches("/// ")
                                            .trim_start_matches("->")
                                            .trim_start_matches("<!-"),
                                    );
                                    string.push('\n')
                                }
                            }

                            println!(
                                "{} -> {}",
                                entry.path().display(),
                                output.clone() + "/" + name
                            );

                            if !Path::new(&output).is_dir() {
                                fs::create_dir(&output).expect("Failed to create man directory");
                            }

                            let mut file = OpenOptions::new()
                                .write(true)
                                .create_new(true)
                                .open(output.clone() + "/" + name)
                                .expect("Failed to create man page");
                            file.write_all(string.as_bytes())
                                .expect("Failed to write man page");
                        }
                    }
                    Err(err) => {
                        println!("docgen: failed to read {}: {}", entry.path().display(), err);
                    }
                }
            }
            Err(err) => {
                println!("docgen: failed to open {}: {}", entry.path().display(), err);
            }
        }
    }
}
