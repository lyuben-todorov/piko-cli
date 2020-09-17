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

const HISTORY_FILE: &str = "linefeed.hst";

fn main() -> io::Result<()> {
    const DEFAULT_PORT: u16 = 8878;

    let connection = TcpStream::connect(SocketAddr::from(
        SocketAddrV4::new(Ipv4Addr::from_str("127.0.0.1").unwrap(), DEFAULT_PORT)
    ).unwrap()).unwrap();

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
            "quit" => break,
            "sub" => {

            }
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
