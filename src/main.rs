use clap::Parser;
use quick_xml::events::Event;
use quick_xml::Writer;
use std::borrow::Cow;
use std::fs;
use std::io::{Cursor, Read};

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct InputOutput {
    /// Input path
    input: String,
    /// Output path
    output: String,
}

fn main() {
    let args = InputOutput::parse();

    let mut file = std::fs::File::open(args.input).expect("Couldn't open file");
    let mut file_str = String::new();
    let _ = file
        .read_to_string(&mut file_str)
        .expect("Couldn't read to file");
    let mut reader = quick_xml::Reader::from_str(&file_str);
    enum State {
        Script,
        Other,
    }
    let mut state = State::Other;
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut saved_script: Option<Cow<[u8]>> = None;

    while let Ok(token) = reader.read_event_unbuffered() {
        state = match (state, &token) {
            (_, Event::Eof) => break,
            (_, event @ Event::Start(ref e)) if e.name().eq(b"script") => {
                let _ = writer.write_event(event);
                State::Script
            }
            (_, event @ Event::End(e)) if e.name().eq(b"script") => {
                let _ = writer.write_event(event);
                State::Other
            }
            (state, event @ Event::Start(_) | event @ Event::End(_)) => {
                let _ = writer.write_event(event);
                state
            }
            (State::Script, Event::CData(e)) => {
                let data = e.clone().into_inner();
                saved_script = Some(data);
                State::Script
            }
            (state, event) => {
                let _ = writer.write_event(event);
                state
            }
        }
    }

    let mut file_writer = Writer::new(Cursor::new(Vec::new()));

    let _ = file_writer.write(
        br#"import { createEffect } from "solid-js";

export const Icon = () => {
"#,
    );

    if let Some(script) = &saved_script {
        let _ = file_writer.write(b" createEffect(() => {");
        let _ = file_writer.write(script.as_ref());
        let _ = file_writer.write(b"});");
    }

    let _ = file_writer.write(
        br#"
    return (
    "#,
    );

    let _ = file_writer.write(&writer.into_inner().into_inner());

    let _ = file_writer.write(
        br#"
    );
};

export default Icon;"#,
    );

    let str = file_writer.into_inner().into_inner();
    let result = String::from_utf8(str).expect("Couldn't convert to string");

    let _ = fs::write(&args.output, result).expect("Couldn't save file");
    println!("Saved file in: {}", args.output);
}
