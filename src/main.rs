use prooftrex::Parser;

fn main() {
    let mut parser = Parser::from_file("test.txt").unwrap();
    parser.parse().unwrap();
}

