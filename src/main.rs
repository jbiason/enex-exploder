use std::env;
use xml::{Event, Parser};
use std::fs::File;
use std::io::prelude::*;
use slug::slugify;

enum CurrentTag {
    Title
}

#[derive(Default)]
struct State {
    tag: Option<CurrentTag>,
    title: Option<String>
}

impl State {
    pub fn new() -> Self {
        Self { tag: None, title: None }
    }

    pub fn change_title(self, title:&str) -> Self {
        Self { title: Some(title.to_string()),
               ..Default::default()
         }
    }

    pub fn change_tag(self, tag:CurrentTag) -> Self {
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
                    "title" => state.change_tag(CurrentTag::Title),
                    "note" => State::new(),     // the start of a note resets everything
                    _ => state
                }
            },
            Event::ElementEnd(_) => {
                state.remove_tag()
            },
            Event::Characters(data) => {
                // println!("Data: {}", data);
                match state.tag {
                    Some(CurrentTag::Title) => {
                        println!("TITLE: {}", slugify(&data));
                        let new_state = state.change_title(&slugify(&data));
                        new_state
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
