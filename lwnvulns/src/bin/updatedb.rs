extern crate lwnvulns;

use std::io::stdin;
use std::io::Read;

fn main() {
    let mut buf = String::new();
    let stdsrc = stdin();
    let mut handle = stdsrc.lock();
    handle.read_to_string(&mut buf).unwrap();

    let tokens = lwnvulns::tokenize::tokenize(buf);
    let ast = lwnvulns::parse::parse(tokens);

    let (_, done) = lwnvulns::transform::partition_completed(ast.unwrap());

    for (_, section) in done.report {
        for issue in section.children {
            print!("{}\n", issue.source.id);
        }
    }
}
