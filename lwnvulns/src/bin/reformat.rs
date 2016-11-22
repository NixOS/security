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

    let (todo, done) = lwnvulns::transform::partition_completed(ast.unwrap());

    if todo.count_issues() == 0 {
        println!("
{todo_doc}
{done_doc}
",
                 todo_doc = lwnvulns::writer::write(&todo),
                 done_doc = lwnvulns::writer::write(&done));
    } else if done.count_issues() == 0 {
        println!("
{todo_doc}
{done_doc}
",
                 todo_doc = lwnvulns::writer::write(&todo),
                 done_doc = lwnvulns::writer::write(&done));
    } else {
        println!("
{todo_doc}

---
## {todo_count} remaining, {done_count} completed

<details><summary><strong>
    CLICK TO EXPAND Show all {done_count} triaged and resolved issues
  </strong></summary>

{done_doc}

</details>

",
                 todo_doc=lwnvulns::writer::write(&todo),
                 done_doc=lwnvulns::writer::write(&done),
                 todo_count=todo.count_issues(),
                 done_count=done.count_issues(),
        );
    }
}
