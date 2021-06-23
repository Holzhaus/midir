extern crate midir;

use std::error::Error;
use std::io::{stdin, stdout, Write};
use midir::{MidiInput, MidiInputPort, MidiOutput, MidiOutputPort, Ignore};

struct MidiDeviceManager {
    input: MidiInput,
    output: MidiOutput,
}

impl MidiDeviceManager {
    pub fn new() -> Result<Self, midir::InitError> {
        let mut input = MidiInput::new("input port watcher")?;
        input.ignore(Ignore::None);
        let output = MidiOutput::new("output port watcher")?;
        Ok(MidiDeviceManager {
            input,
            output,
        })
    }

    pub fn ports(&self) -> Vec<(String, MidiInputPort, MidiOutputPort)> {
        self.input.ports().into_iter().filter_map(|input_port| {
            if let Some(port_name) = self.input.port_name(&input_port).ok() {
                if let Some(output_port) = self.output_port(&port_name) {
                    return Some((port_name, input_port, output_port));
                }
            }
            None
        }).collect()
    }

    fn input_port(&self, port_name: &str) -> Option<MidiInputPort> {
        self.input.ports().into_iter().find(|port| self.input.port_name(&port).map_or(false, |name| name == port_name))
    }

    fn output_port(&self, port_name: &str) -> Option<MidiOutputPort> {
        self.output.ports().into_iter().find(|port| self.output.port_name(&port).map_or(false, |name| name == port_name))
    }

    pub fn is_available(&self, port_name: &str) -> bool {
        let input_port = self.input_port(port_name);
        let output_port = self.output_port(port_name);
        input_port.is_some() && output_port.is_some()
    }
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let manager = MidiDeviceManager::new()?;
    let ports = manager.ports();
    let port_name = match ports.len() {
        0 => return Err("no port found".into()),
        1 => {
            println!("Choosing the only available port: {}", &ports[0].0);
            &ports[0].0
        },
        _ => {
            println!("\nAvailable ports:");
            for (i, (name, _, _)) in ports.iter().enumerate() {
                println!("{}: {}", i, name);
            }
            print!("Please select port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            &ports.get(input.trim().parse::<usize>()?)
                     .ok_or("invalid port selected")?.0
        }
    };

    println!("Starting endless loop, press CTRL-C to exit...");

    let mut last_state = None;
    loop {
        let current_state = manager.is_available(port_name);
        if last_state != Some(current_state) {
            if current_state {
                println!("{}: available", port_name);
            } else {
                println!("{}: unavailable", port_name);
            }
            last_state = Some(current_state);
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
}
