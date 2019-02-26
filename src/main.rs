use std::env;
use xml::{Event, Parser};
use std::fs::File;
use std::io::prelude::*;
use slug::slugify;

enum State {
    Title
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
    
    parser.fold(None, {|state:Option<State>, element| {
        match element.unwrap() {
            Event::ElementStart(tag) => {
                match tag.name.as_ref() {
                    "title" => Some(State::Title),
                    _ => None
                }
            },
            Event::ElementEnd(_) => {
                None    // ending a tag always remove the state
            },
            Event::Characters(data) => {
                // println!("Data: {}", data);
                match state {
                    Some(State::Title) => {
                        println!("TITLE: {}", slugify(data));
                        None
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
