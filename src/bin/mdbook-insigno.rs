extern crate clap;
extern crate mdbook;
extern crate serde_json;
extern crate regex;

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use regex::{Regex, Captures};
use clap::{App, Arg, ArgMatches, SubCommand};
use std::io;
use std::process::{Command, Stdio,};
use std::io::{Write, Stderr};
use std::fs;
use std::process;
use mdbook::book::{Book,BookItem,Chapter};
use mdbook::errors::{Error,Result};
use mdbook::preprocess::{Preprocessor, PreprocessorContext, CmdPreprocessor};

pub fn make_app() -> App<'static, 'static> {
    App::new("dot-preprocessor")
        .about("A mdbook preprocessor so genereate c# plantuml.")
        .subcommand(
            SubCommand::with_name("supports")
                .arg(Arg::with_name("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();

    let mdd = MdbookDot::new();

    //We support everthing
    if let Some(_) = matches.subcommand_matches("supports") {
        process::exit(0);
    }

    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin()).unwrap();
    let uml_cmd = match ctx.config.get("insigno-uml-cmd") {
        Some(val) => {
            match val.as_str() {
                Some(str_val) => {str_val},
                None => {""}
            }
        }
        None => {""}
    };
    uml_setup(GIT_DIR, GIT_REMOTE, UML_DIR);
    let processed_book = mdd.run(&ctx,book).unwrap();
    serde_json::to_writer(io::stdout(), &processed_book).unwrap();
}

const UML_DIR: &str = "/tmp/uml/";
const GIT_DIR: &str = "/tmp/insigno/";
const GIT_REMOTE: &str = "git@github.com:spacycoder/VectorMap.git";
const NAME: &str = "mdbook-puml-gen";

struct MdbookDot;

impl MdbookDot {
    pub fn new() -> MdbookDot {
        MdbookDot
    }
}

impl Preprocessor for MdbookDot {
    fn name(&self) -> &str {
       NAME
    }
    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        eprintln!("Running '{}' preprocessor",self.name());
        let mut num_replaced_items = 0;
        process(&mut book.sections, &mut num_replaced_items).unwrap();
        Ok(book)
    }
    fn supports_renderer(&self, renderer: &str) -> bool {
        true
    }
}

fn process <'a,I>(items: I, num_replaced_items: &mut usize) -> Result<()> where
    I : IntoIterator<Item = &'a mut BookItem> + 'a,
{
    let re = Regex::new(r"\$(?P<cmd>\w+)\((?P<arg>[^)]+)\)").unwrap();
    for item in items {
        if let BookItem::Chapter(ref mut chapter) = *item {
            process(&mut chapter.sub_items, num_replaced_items).unwrap();
            chapter.content = re.replace_all(chapter.content.as_str(), | caps: &Captures | {
                match &caps["cmd"] {
                    "uml" => {
                        eprintln!("Getting uml for '{}'", &caps["arg"]);
                        match get_uml(&caps["arg"]) {
                            Ok(data) => {
                                data
                            },
                            Err(e) => {
                                eprintln!("Failed to get uml '{}'", e);
                                "".to_string()
                            },
                        }
                    },
                    "src" => {
                        eprintln!("Getting source for '{}'",&caps["arg"]);
                        match get_src(&caps["arg"]) {
                            Ok(data) => {
                                data
                            },
                            Err(e) => {
                                eprintln!("Failed to get uml '{}'", e);
                                "".to_string()
                            },
                        }
                    }
                    _ => {eprintln!("unkown command '{}'", &caps["cmd"]); "".to_string()}
                }
            }).to_string();

        }
    }
    Ok(())
}

fn uml_setup(path:&str, remote:&str, uml_dir:&str) {
    let git_dir = Path::new(path);
    if git_dir.exists() {
        eprintln!("Updating git repo '{}'", remote);
        let mut git = Command::new("/usr/bin/git")
            .args(&["-C", path, "pull", "origin", "master"]).stdout(Stdio::null())
            .spawn().expect("Git failed");
        git.wait_with_output().expect("Failed to wait for git");
    }
    else {
        eprintln!("Cloning git repo '{}'", remote);
        let mut git = Command::new("/usr/bin/git")
            .args(&["clone", remote, path]).stdout(Stdio::null())
            .spawn().expect("Git failed");
        git.wait_with_output().expect("Failed to wait for git");
    }
    eprintln!("Building uml files");
    let mut git = Command::new("/root/.dotnet/tools/puml-gen")
        .args(&[path, "-dir", uml_dir]).stdout(Stdio::null())
        .spawn().expect("puml-gen falied");
    git.wait_with_output().expect("Failed to wait for git");
}


fn get_src(path:&str) -> Result<String> {
    let path = format!("{}{}.cs", GIT_DIR, path);
    //eprintln!("{}",path);
    let mut file = File::open(path)?;
    let mut data = String::from("\n```cs\n");
    file.read_to_string(&mut data)?;
    data.push_str("\n```\n");
    return Ok(data);
}

fn get_uml(path:&str) -> Result<String> {
    let path = format!("{}{}.puml", UML_DIR, path);
    //eprintln!("{}",path);
    let mut file = File::open(path)?;
    let mut data = String::from("\n```plantuml\n");
    file.read_to_string(&mut data)?;
    data.push_str("\n```\n");
    return Ok(data);
}
