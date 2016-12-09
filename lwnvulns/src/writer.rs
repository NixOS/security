

use parse::Document;
use tokenize::Token;


pub fn write(input: &Document) -> String {
    let mut ret = "".to_string();
    for token in &input.preamble {
        if let &Token::Preamble(ref line) = token {
            ret = ret + &line + "\n";
        } else {
            panic!("lol no other tokens here?");
        }
    }

    let mut reports: Vec<String> = input.report.keys().map(|key| key.clone()).collect();
    reports.sort();
    for key in reports {
        let section = input.report.get(&key).unwrap();
        let mut header = section.header.clone();
        header.issue_count = section.children.len() as i32;

        ret = ret + &header.to_string() + "\n";

        for issue in &section.children {
            ret = ret + &issue.to_string() + "\n";
        }

        ret = ret + "\n";
    }


    return ret;
}

#[cfg(test)]
mod tests {
    use parse::{Document, Section};
    use tokenize::{Token, Header, Issue, SourceLink};
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_exports_back() {
        let mut input = Document {
            preamble: vec![Token::Preamble("this".to_string()),
                           Token::Preamble("is".to_string()),
                           Token::Preamble("preamble".to_string()),
                           Token::Preamble("### Here too! ( :) )".to_string())],
            report: HashMap::new(),
        };
        input.report.insert("B This is a header too".to_string(),
                            Section {
                                header: Header {
                                    package: "B This is a header too".to_string(),
                                    issue_count: 2,
                                    notes: Some("lol".to_string()),
                                },
                                children: vec![Issue {
                                                   complete: false,
                                                   content: "not done".to_string(),
                                                   source: SourceLink {
                                                       id: "not done".to_string(),
                                                       url: "http://not done".to_string(),
                                                   },
                                               },
                                               Issue {
                                                   complete: true,
                                                   content: "done".to_string(),
                                                   source: SourceLink {
                                                       id: "done".to_string(),
                                                       url: "http://done".to_string(),
                                                   },
                                               }],
                            });

        input.report.insert("A This is a header".to_string(),
                            Section {
                                header: Header {
                                    package: "A This is a header".to_string(),
                                    issue_count: 0,
                                    notes: None,
                                },
                                children: vec![],
                            });

        let expect = "this
is
preamble
### Here too! ( :) )
### A This is a header (0 issues)

### B This is a header too (2 issues) lol
 - [ ] [`#not done`](http://not done) not done
 - [x] [`#done`](http://done) done

";

        assert_eq!(write(&input), expect);

    }
}
