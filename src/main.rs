use std::env;
use xml::{Event, Parser};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::vec::Vec;
use slug::slugify;
use base64::decode;

#[derive(Debug)]
enum CurrentTag {
    Title,
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
    filename: Option<String>,
    data: Vec<u8>
}

impl State {
    pub fn new() -> Self {
        Self { tag: None, title: None, filename: None, data: Vec::new() }
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

    pub fn with_data(self, data:Vec<u8>) -> Self {
        Self { data: data,
               ..self }
    }

    pub fn remove_tag(self) -> Self {
        Self { tag: None,
               ..self }
    }

    pub fn remove_data(self) -> Self {
        Self { data: Vec::new(),
               ..self }
    }
}

fn create_note_storage(title: &str) -> String {
    let slug = slugify(title);
    std::fs::create_dir_all(Path::new("data").join(slug.as_str())).unwrap();
    slug
}

fn open_tag(current_state: State, tag: &str) -> State {
    match tag {
        "title" => current_state.with_tag(CurrentTag::Title),
        "data" => current_state.with_tag(CurrentTag::ResourceData),
        "content" => current_state.with_tag(CurrentTag::Content),
        "resource" => current_state.with_tag(CurrentTag::Resource),
        "resource-attributes" => current_state.with_tag(CurrentTag::ResourceAttributes),
        "file-name" => current_state.with_tag(CurrentTag::ResourceAttributesFilename),
        "note" => State::new(),
        _ => current_state
    }
}

fn dump_resource(current_state: &State) -> () {
    let unnamed = String::from("content");
    let content_storage = Path::new("data")
        .join(current_state.title.as_ref().unwrap())
        .join(current_state.filename.as_ref().unwrap_or(&unnamed));
    let content = base64::decode(&current_state.data).unwrap();

    println!("Will save {:?}", content_storage);

    let mut target = File::create(content_storage).unwrap();
    target.write_all(&content).unwrap();
}

fn close_tag(current_state: State, tag: &str) -> State {
    match tag {
        "resource" => {
            dump_resource(&current_state);
            current_state.remove_data()
        },
        _ => current_state.remove_tag()
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
        println!("State: {:?}", state);

        match element.unwrap() {
            Event::ElementStart(tag) => open_tag(state, tag.name.as_ref()),
            Event::ElementEnd(tag) => close_tag(state, tag.name.as_ref()),
            Event::Characters(data) => {
                match state.tag {
                    Some(CurrentTag::Title) => state.with_title(create_note_storage(&data)),
                    Some(CurrentTag::ResourceData) => state.with_data(
                        data
                            .into_bytes()
                            .iter()
                            .filter(|&x| *x != 13 && *x != 10)  // remove new lines, it is base 64, after all
                            .map(|&x| x)
                            .collect()
                    ),
                    Some(CurrentTag::ResourceAttributesFilename) => state.with_filename(data),
                    _ => state
                }
            },
            _ => state
        }
    }});
}
