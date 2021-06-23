extern crate midir;

use std::error::Error;
use std::io::{stdin, stdout, Write};
use midir::{MidiInput, MidiInputPort, MidiInputConnection, MidiOutput, MidiOutputPort, MidiOutputConnection, Ignore};

struct MidiDevice {
    name: String,
    input_port: MidiInputPort,
    output_port: MidiOutputPort,
    input_connection: Option<MidiInputConnection<()>>,
}

impl MidiDevice {
    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    fn disconnect_input(&mut self) -> bool {
        let conn = std::mem::replace(&mut self.input_connection, None);
        match conn {
            Some(connection) => {
                connection.close();
                true
            },
            _ => false,
        }
    }

    pub fn connect_input(&mut self) -> bool {
        if let Some(conn) = &self.input_connection {
            return false;
        }
        let input_port = self.input_port.clone();
        let input = MidiInput::new("input").unwrap();
        self.input_connection = Some(input.connect(&input_port, "midir-read-input", move |stamp, message, _| {
            println!("{}: {:?} (len = {})", stamp, message, message.len());
        }, ()).expect("failed to connect"));
        true
    }
}

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

    pub fn ports(&self) -> Vec<MidiDevice> {
        self.input.ports().into_iter().filter_map(|input_port| {
            if let Some(name) = self.input.port_name(&input_port).ok() {
                if let Some(output_port) = self.output_port(&name) {
                    let input_connection = None;
                    return Some(MidiDevice { name, input_port, output_port, input_connection });
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
    let mut ports = manager.ports();
    let mut device = match ports.len() {
        0 => return Err("no port found".into()),
        1 => {
            println!("Choosing the only available port: {}", &ports[0].name);
            ports.remove(0)
        },
        _ => {
            println!("\nAvailable ports:");
            for (i, device) in ports.iter().enumerate() {
                println!("{}: {}", i, device.name);
            }
            print!("Please select port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            ports.remove(input.trim().parse::<usize>()?)
        }
    };

    println!("Starting endless loop, press CTRL-C to exit...");

    device.connect_input();

    let mut last_state = None;
    loop {
        let current_state = manager.is_available(&device.name);
        if last_state != Some(current_state) {
            if current_state {
                println!("{}: available", device.name);
                device.connect_input();
            } else {
                println!("{}: unavailable", device.name);
                device.disconnect_input();
            }
            last_state = Some(current_state);
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
}
