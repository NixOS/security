use regex::Regex;
use std::hash::{Hash, Hasher};

#[derive(Debug,PartialEq)]
pub enum Token {
    Preamble(String),
    Header(Header),
    Issue(Issue),
}

#[derive(Debug,PartialEq,Clone)]
pub struct Header {
    pub package: String,
    pub issue_count: i32,
    pub notes: Option<String>,
}

#[derive(Debug,PartialEq)]
pub struct Issue {
    pub complete: bool,
    pub source: SourceLink,
    pub content: String,
}

#[derive(Debug,PartialEq)]
pub struct SourceLink {
    pub url: String,
    pub id: String,
}

impl Hash for Header {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.package.hash(state);
    }
}

impl Header {
    pub fn to_string(&self) -> String {
        if let Some(_) = self.notes {
            if self.issue_count == 1 {
                return format!("### {} ({} issue) {}",
                               self.package,
                               self.issue_count,
                               self.notes.clone().unwrap().clone())
                    .to_string();
            } else {
                return format!("### {} ({} issues) {}",
                               self.package,
                               self.issue_count,
                               self.notes.clone().unwrap().clone())
                    .to_string();
            }
        } else {
            if self.issue_count == 1 {
                return format!("### {} ({} issue)", self.package, self.issue_count).to_string();
            } else {
                return format!("### {} ({} issues)", self.package, self.issue_count).to_string();
            }
        }
    }
}

impl Issue {
    pub fn to_string(&self) -> String {
        if self.complete {
            format!(" - [x] [`#{}`]({}) {}",
                    self.source.id,
                    self.source.url,
                    self.content)
                .to_string()
        } else {
            format!(" - [ ] [`#{}`]({}) {}",
                    self.source.id,
                    self.source.url,
                    self.content)
                .to_string()
        }
    }
}

impl Token {
    pub fn to_string(&self) -> String {
        match *self {
            Token::Preamble(ref res) => res.to_string(),
            Token::Header(ref header) => header.to_string(),
            Token::Issue(ref issue) => issue.to_string(),
        }
    }
}

fn parse_header(line: String) -> Option<Token> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"^### (.*) \((\d+) issues?\) ?(.+)?$"
        ).unwrap();
    }
    if let Some(capture) = RE.captures(&line) {
        if let Some(package) = capture.at(1) {
            if let Some(count) = capture.at(2) {
                if let Ok(count) = count.parse::<i32>() {
                    let mut notes: Option<String> = None;

                    if let Some(found_notes) = capture.at(3) {
                        notes = Some(found_notes.to_string())
                    }

                    return Some(Token::Header(Header {
                        package: package.to_string(),
                        issue_count: count,
                        notes: notes,
                    }));
                }
            }
        }
    }

    return None;
}

fn parse_issue(line: String) -> Option<Token> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"^ ?- \[(x| )\] \[`#(.*)`\]\((.*)\) (.*)$"
        ).unwrap();
    }
    if let Some(capture) = RE.captures(&line) {
        if let Some(complete) = capture.at(1) {
            let complete = complete == "x";
            return Some(Token::Issue(Issue {
                complete: complete,
                source: SourceLink {
                    id: capture.at(2).unwrap().to_string(),
                    url: capture.at(3).unwrap().to_string(),
                },
                content: capture.at(4).unwrap().to_string(),
            }));
        }
    }

    return None;
}

pub fn tokenize(inputs: String) -> Vec<Token> {
    let mut ret: Vec<Token> = vec![];
    let mut in_preamble: bool = true;

    for line in inputs.lines() {
        if let Some(header) = parse_header(line.to_string()) {
            in_preamble = false;
            ret.push(header)
        } else if in_preamble {
            ret.push(Token::Preamble(line.to_string()));
        } else if let Some(issue) = parse_issue(line.to_string()) {
            ret.push(issue)
        } else {

        }

    }
    return ret;
}

#[test]
fn test_tokenize() {
    let input = "this
is
garbage
### Here too! ( :) )
### This is a header (0 issues)
### This is a header too (1 issue) lol
garbage1
 - [ ] [`#123456`](http://foo...) not done
garbage2
 - [x] [`#7890`](http://bar...) done
garbage3
";
    let expect = vec![
        Token::Preamble("this".to_string()),
        Token::Preamble("is".to_string()),
        Token::Preamble("garbage".to_string()),
        Token::Preamble("### Here too! ( :) )".to_string()),
        Token::Header(Header {
            package: "This is a header".to_string(),
            issue_count: 0,
            notes: None
        }),
        Token::Header(Header {
            package: "This is a header too".to_string(),
            issue_count: 1,
            notes: Some("lol".to_string())
        }),
        Token::Issue(Issue {
            complete: false,
             source: SourceLink {
                url: "http://foo...".to_string(),
                id: "123456".to_string()
            },
            content: "not done".to_string(),
        }),
        Token::Issue(Issue {
            complete: true,
            source: SourceLink {
                url: "http://bar...".to_string(),
                id: "7890".to_string()
            },
            content: "done".to_string(),
        }),
    ];

    assert_eq!(tokenize(input.to_string()), expect);
}
