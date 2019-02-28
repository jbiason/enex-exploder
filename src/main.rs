use std::env;
use xml::{Event, Parser};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use slug::slugify;

enum CurrentTag {
    Title,
    Data
}

#[derive(Default)]
struct State {
    tag: Option<CurrentTag>,
    title: Option<String>,
    filename: Option<String>
}

impl State {
    pub fn new() -> Self {
        Self { tag: None, title: None, filename: None }
    }

    pub fn with_title(self, title:String) -> Self {
        Self { title: Some(title.to_string()),
               ..Default::default() }
    }

    pub fn with_filename(self, filename: String) -> Self {
        Self { filename: Some(filename),
               ..Default::default() }
    }

    pub fn with_tag(self, tag:CurrentTag) -> Self {
        Self { tag: Some(tag),
               ..Default::default() }
    }

    pub fn remove_tag(self) -> Self {
        Self { tag: None,
               ..Default::default() }
    }
}

fn main() {
    let args:Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Required: filename.");
        std::process::exit(2);
    }
    
    let filename = &args[1];
        
    println!("Will process {}", filename);
    
    // Need to find a way to load small chunks and feed it to the parser while parsing.
    // (E.g., load 1024 bytes, feed it to the parser and, if the parser can't continue,
    //  feed more data, till the end of file).
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
        
    let mut parser = Parser::new();
    parser.feed_str(&contents);
    
    parser.fold(State::new(), {|state:State, element| {
        match element.unwrap() {
            Event::ElementStart(tag) => {
                match tag.name.as_ref() {
                    "title" => state.with_tag(CurrentTag::Title),
                    "data" => state.with_tag(CurrentTag::Data),
                    "note" => State::new(),     // the start of a note resets everything
                    _ => state
                }
            },

            Event::ElementEnd(_) => {
                // whatever tag we were following, it is not there anymore.
                state.remove_tag()
            },

            Event::Characters(data) => {
                match state.tag {
                    Some(CurrentTag::Title) => {
                        let slug = slugify(data);
                        println!("TITLE: {}", slug);
                        std::fs::create_dir_all(Path::new(slug.as_str())).unwrap();
                        state.with_title(slug)
                    },
                    Some(CurrentTag::Data) => {
                        let title = state.title.unwrap().to_string();
                        let _filename = Path::new(&title);

                        state
                    },
                    _ => state
                }
            },
            Event::CDATA(_) => {
                state
            },
            _ => state
        }
    }});
}
