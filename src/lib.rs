use std::fs::File;
use std::io::{prelude::*, BufReader};
use Command::*;

pub struct Parser {
    source: String,
    cursor: usize,
}

impl Parser {
    pub fn from_file(path: &str) -> Result<Parser, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        Ok(Parser {
            source: contents,
            cursor: 0,
        })
    }

    pub fn parse(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let theorem_line = self.source.lines().find(|s| s.starts_with("Theorem"))
            .expect("Failed to detect theorem")
            .replace(|c| c == '.' || c == ' ', "");
        let theorem = theorem_line
            .split(':')
            .rev()
            .nth(0)
            .unwrap();

        let mut root = Inference::new(Theorem, theorem);

        self.skip_while(|s| s.starts_with("Proof"));
        self.recursively_parse(&mut root, 1);

        dbg!(root);

        Ok(())
    }

    fn recursively_parse(&mut self, parent: &mut Inference, num_infers: usize) -> () {
        // Okay let's write this out....
        // for each inference to make (num_infer),
        // parse a line's command type and parameters,
        // if its a "exact", add it to the parent and return
        for _ in 0..num_infers {
            let line = self.next().expect("No more lines available");
            let mut infer = Self::parse_line(line).expect("Failed to parse line");
            let sub_num_infer = infer.cmd.num_infers();

            self.recursively_parse(&mut infer, sub_num_infer);

            parent.add_infer(infer);
        }
    }

    fn skip(&mut self, n: usize) -> () {
        self.cursor += n;
    }

    fn skip_while(&mut self, f: fn(&str) -> bool) -> () {
        while let Some(line) = self.next() {
            if f(line) {
                break;
            }
        }
    }

    fn next(&mut self) -> Option<&str> {
        self.skip(1);
        self.source.lines().nth(self.cursor)
    }

    fn parse_line(line: &str) -> Result<Inference, &'static str> {
        let i = line.find(|c: char| c == ' ' || c == '.').unwrap_or(0);
        let cmd = Command::from_str(&line[0..i]);

        let parts: Vec<&str> = line[..line.len() - 1].split(' ').skip(1).collect();

        if let Some(pn) = cmd.num_params() {
            if parts.len() != pn {
                return Err("Incorrect amount of parameters provided to command");
            }
        }

        let infer = match cmd {
            ImpI | Exact | PBC => {
                let mut inf = Inference::new(cmd, "body tba...");
                inf.add_label(parts[0]);
                inf
            },
            _ => Inference::new(cmd, ""),
        };

        Ok(infer)
    }
}

#[derive(Debug)]
pub struct Inference {
    cmd: Command,
    body: String,
    infers: Vec<Inference>,
    labels: Vec<String>,
}

impl Inference {
    fn new<S: Into<String>>(cmd: Command, body: S) -> Self {
        Self {
            cmd,
            body: body.into(),
            infers: Vec::new(),
            labels: Vec::new(),
        }
    }

    fn add_label<S: Into<String>>(&mut self, label: S) -> () {
        self.labels.push(label.into());
    }

    fn add_infer(&mut self, infer: Inference) -> () {
        self.infers.push(infer);
    }
}

#[derive(Debug)]
enum Command {
    Parameter,
    Theorem,
    Proof,
    ConI,
    ImpI,
    ImpE,
    Exact,
    PBC,
    NegE,
    NegNegE,
    Qed,
    Unknown,
}

impl Command {
    fn from_str(s: &str) -> Command {
        match s {
            "Parameter" => Parameter,
            "Theorem" => Theorem,
            "Proof" => Proof,
            "con_i" => ConI,
            "imp_i" => ImpI,
            "imp_e" => ImpE,
            "exact" => Exact,
            "PBC" => PBC,
            "neg_e" => NegE,
            "negneg_e" => NegNegE,
            "Qed" => Qed,
            _ => Unknown,
        }
    }

    fn num_params(&self) -> Option<usize> {
        match self {
            Parameter | Theorem | Unknown => None,
            Proof | NegNegE | Qed | ConI => Some(0),
            ImpI | ImpE | Exact | PBC | NegE => Some(1),
        }
    }

    fn num_infers(&self) -> usize {
        match self {
            ConI | ImpE | NegE => 2,
            ImpI | PBC | NegNegE => 1,
            _ => 0,
        }
    }
}

