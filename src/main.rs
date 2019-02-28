use std::env;
use xml::{Event, Parser};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use slug::slugify;

#[derive(Debug)]
enum CurrentTag {
    Title,
    Data,
    Content,
    Resource,
    ResourceData,
    ResourceAttributes,
    ResourceAttributesFilename
}

#[derive(Debug)]
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
               ..self }
    }

    pub fn with_filename(self, filename: String) -> Self {
        Self { filename: Some(filename),
               ..self }
    }

    pub fn with_tag(self, tag:CurrentTag) -> Self {
        Self { tag: Some(tag),
               ..self }
    }

    pub fn remove_tag(self) -> Self {
        Self { tag: None,
               ..self }
    }
}

fn create_note_storage(title: &str) -> String {
    let slug = slugify(title);
    println!("TITLE: {}", slug);
    std::fs::create_dir_all(Path::new("data").join(slug.as_str())).unwrap();
    slug
}

fn open_tag(current_state: State, tag: &str) -> State {
    match tag {
        "title" => current_state.with_tag(CurrentTag::Title),
        "data" => current_state.with_tag(CurrentTag::Data),
        "content" => current_state.with_tag(CurrentTag::Content),
        "resource" => current_state.with_tag(CurrentTag::Resource),
        "resource-attributes" => current_state.with_tag(CurrentTag::ResourceAttributes),
        "file-name" => current_state.with_tag(CurrentTag::ResourceAttributesFilename),
        "note" => State::new(),
        _ => current_state
    }
}

fn close_tag(current_state: State, _tag: &str) -> State {
    current_state.remove_tag()
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
        println!("State: {:?}", state);

        match element.unwrap() {
            Event::ElementStart(tag) => open_tag(state, tag.name.as_ref()),
            Event::ElementEnd(tag) => close_tag(state, tag.name.as_ref()),
            // Event::CDATA(_) => state,
            Event::Characters(data) => {
                println!("Data");

                match state.tag {
                    Some(CurrentTag::Title) => state.with_title(create_note_storage(&data)),
                    Some(CurrentTag::Data) => state,
                    Some(CurrentTag::ResourceAttributesFilename) => state.with_filename(data),
                    _ => state
                }
            },
            _ => state
        }
    }});
}
