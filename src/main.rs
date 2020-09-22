use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rand::{Rng, thread_rng};

use linefeed::{Interface, Prompter, ReadResult};
use linefeed::chars::escape_sequence;
use linefeed::command::COMMANDS;
use linefeed::complete::{Completer, Completion};
use linefeed::inputrc::parse_text;
use linefeed::terminal::Terminal;
use std::net::{TcpStream, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::io::{Write, Read};
use piko::client::{ClientReq, ClientRes};
use byteorder::{WriteBytesExt, ReadBytesExt};

const HISTORY_FILE: &str = "linefeed.hst";

pub fn write_req(stream: &mut TcpStream, client_req: ClientReq) {
    let req = serde_cbor::to_vec(&client_req).unwrap();

    let size = req.len();

    stream.write_u8(size as u8).unwrap();
    stream.write_all(req.as_slice()).unwrap();
}


pub fn read_res(stream: &mut TcpStream) -> ClientRes {
    let size = stream.read_u8().unwrap();
    let mut buf = vec![0u8; size as usize];
    stream.read_exact(&mut buf).unwrap();
    let res: ClientRes = serde_cbor::from_slice(buf.as_slice()).unwrap();
    res
}

fn input(address: &SocketAddr, req: ClientReq) -> ClientRes {
    let mut stream = TcpStream::connect(address).unwrap();

    write_req(&mut stream, req);

    read_res(&mut stream)
}

fn main() -> io::Result<()> {
    const DEFAULT_PORT: u16 = 8878;
    const CLIENT_ID: u64 = 1234;

    let address = SocketAddr::from(SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1").unwrap(), DEFAULT_PORT));

    let interface = Arc::new(Interface::new("piko")?);

    println!("Enter \"help\" for a list of commands.");
    println!("Press Ctrl-D or enter \"quit\" to exit.");
    println!();

    interface.set_completer(Arc::new(DemoCompleter));
    interface.set_prompt("piko> ")?;

    if let Err(e) = interface.load_history(HISTORY_FILE) {
        if e.kind() == io::ErrorKind::NotFound {
            println!("History file {} doesn't exist, not loading history.", HISTORY_FILE);
        } else {
            eprintln!("Could not load history file {}: {}", HISTORY_FILE, e);
        }
    }

    while let ReadResult::Input(line) = interface.read_line()? {
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        }

        let (cmd, args) = split_first_word(&line);

        match cmd {
            "help" => {
                println!("piko-cli commands:");
                println!();
                for &(cmd, help) in NODE_COMMANDS {
                    println!("  {:15} - {}", cmd, help);
                }
                println!();
            }
            "list-commands" => {
                for cmd in COMMANDS {
                    println!("{}", cmd);
                }
            }
            "pub" => {

            }
            "sub" => {
                let req = ClientReq::sub(CLIENT_ID);

                let res = input(&address, req);
                match res {
                    ClientRes::Success { message, bytes } => {
                        println!("{}", message);
                    }
                    _ => {}
                }
            }
            "quit" => break,

            _ => println!("Unknown command: {:?}", line)
        }
    }
    println!("Goodbye.");

    Ok(())
}

fn split_first_word(s: &str) -> (&str, &str) {
    let s = s.trim();

    match s.find(|ch: char| ch.is_whitespace()) {
        Some(pos) => (&s[..pos], s[pos..].trim_start()),
        None => (s, "")
    }
}

static NODE_COMMANDS: &[(&str, &str)] = &[
    ("help", "You're looking at it"),
    ("list-commands", "List command names"),
    ("quit", "Quit"),
    ("sub", "Subscribe to cluster"),
    ("unsub", "Unsubscribe from cluster"),
    ("pub", "Publish to cluster"),
    ("poll", "Poll you message queue from cluster")
];

struct DemoCompleter;

impl<Term: Terminal> Completer<Term> for DemoCompleter {
    fn complete(&self, word: &str, prompter: &Prompter<Term>,
                start: usize, _end: usize) -> Option<Vec<Completion>> {
        let line = prompter.buffer();

        let mut words = line[..start].split_whitespace();

        match words.next() {
            // Complete command name
            None => {
                let mut compls = Vec::new();

                for &(cmd, _) in NODE_COMMANDS {
                    if cmd.starts_with(word) {
                        compls.push(Completion::simple(cmd.to_owned()));
                    }
                }

                Some(compls)
            }
            _ => None
        }
    }
}
