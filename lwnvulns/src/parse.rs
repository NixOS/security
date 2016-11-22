use tokenize::{Token, Header, Issue};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};


#[derive(Debug,PartialEq)]
pub struct Document {
    pub preamble: Vec<Token>,
    pub report: HashMap<String, Section>,
}

#[derive(Debug,PartialEq)]
pub struct Section {
    pub header: Header,
    pub children: Vec<Issue>,
}

impl Hash for Section {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.header.hash(state);
    }
}

impl Document {
    pub fn count_issues(&self) -> usize {
        let mut sum: usize = 0;

        for value in self.report.values() {
            sum += value.children.len();
        }

        return sum;
    }
}

pub fn parse(mut input: Vec<Token>) -> Result<Document, String> {
    let mut doc = Document {
        preamble: vec![],
        report: HashMap::new(),
    };

    input.reverse();

    let mut token = input.pop();
    while let Some(Token::Preamble(_)) = token {
        doc.preamble.push(token.unwrap());
        token = input.pop();
    }

    while let Some(Token::Header(mut header)) = token {
        header.issue_count = 0;
        let mut section = doc.report.entry(header.package.to_string()).or_insert(Section {
            header: header,
            children: vec![],
        });

        token = input.pop();

        loop {
            match token {
                Some(Token::Issue(issue)) => {
                    section.children.push(issue);
                    section.header.issue_count += 1;
                    token = input.pop();
                }
                _ => {
                    break;
                }
            }
        }
    }

    return Ok(doc);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokenize::{Token, Header, Issue, SourceLink};
    use std::collections::HashMap;

    #[test]
    fn test_parse() {
        let input = vec![
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
                notes: Some(" lol".to_string())
            }),
            Token::Issue(Issue {
                complete: false,
                content: "not done".to_string(),
                source: SourceLink {
                    id: "1".to_string(),
                    url: "http://1".to_string(),
                },
            }),
            Token::Issue(Issue {
                complete: true,
                content: "done".to_string(),
                source: SourceLink {
                    id: "2".to_string(),
                    url: "http://2".to_string(),
                },

            }),
        ];

        let mut expect = Document {
            preamble: vec![
                Token::Preamble("this".to_string()),
                Token::Preamble("is".to_string()),
                Token::Preamble("garbage".to_string()),
                Token::Preamble("### Here too! ( :) )".to_string()),
            ],
            report: HashMap::new(),
        };

        expect.report.insert("This is a header".to_string(),
                             Section {
                                 header: Header {
                                     package: "This is a header".to_string(),
                                     issue_count: 0,
                                     notes: None,
                                 },
                                 children: vec![],
                             });

        expect.report.insert("This is a header too".to_string(),
                             Section {
                                 header: Header {
                                     package: "This is a header too".to_string(),
                                     issue_count: 2,
                                     notes: Some(" lol".to_string()),
                                 },
                                 children: vec![
                                     Issue {
                                         complete: false,
                                         content: "not done".to_string(),
                                         source: SourceLink {
                                             id: "1".to_string(),
                                             url: "http://1".to_string(),
                                         },

                                     },
                                     Issue {
                                         complete: true,
                                         content: "done".to_string(),
                                         source: SourceLink {
                                             id: "2".to_string(),
                                             url: "http://2".to_string(),
                                         },

                                     },
                                 ],
                             });

        assert_eq!(parse(input), Ok(expect));

    }

    #[test]
    fn test_parse_duplicated_headers() {
        let input = vec![
            Token::Header(Header {
                package: "PackageA".to_string(),
                issue_count: 1,
                notes: Some("lol".to_string())
            }),
            Token::Issue(Issue {
                complete: false,
                content: "not done".to_string(),
                source: SourceLink {
                    id: "3".to_string(),
                    url: "http://3".to_string(),
                },

            }),
            Token::Header(Header {
                package: "PackageA".to_string(),
                issue_count: 1,
                notes: None
            }),
            Token::Issue(Issue {
                complete: true,
                content: "done".to_string(),
                source: SourceLink {
                    id: "4".to_string(),
                    url: "http://4".to_string(),
                },

            }),
        ];

        let mut expect = Document {
            preamble: vec![],
            report: HashMap::new(),
        };
        expect.report.insert("PackageA".to_string(),
                             Section {
                                 header: Header {
                                     package: "PackageA".to_string(),
                                     issue_count: 2,
                                     notes: Some("lol".to_string()),
                                 },
                                 children: vec![
                                     Issue{
                                         complete: false,
                                         content: "not done".to_string(),
                                         source: SourceLink {
                                             id: "3".to_string(),
                                             url: "http://3".to_string(),
                                         },
                                     },
                                     Issue {
                                         complete: true,
                                         content: "done".to_string(),
                                         source: SourceLink {
                                             id: "4".to_string(),
                                             url: "http://4".to_string(),
                                         },
                                     },
                             ],
                             });

        assert_eq!(parse(input), Ok(expect));

    }


    #[test]
    fn test_count_issues() {
        let mut doc = Document {
            preamble: vec![],
            report: HashMap::new(),
        };
        doc.report.insert("PackageA".to_string(),
                          Section {
                              header: Header {
                                  package: "PackageA".to_string(),
                                  issue_count: 2,
                                  notes: Some("lol".to_string()),
                              },
                              children: vec![
                                  Issue{
                                      complete: false,
                                      content: "not done".to_string(),
                                      source: SourceLink {
                                          id: "5".to_string(),
                                          url: "http://5".to_string(),
                                      },
                                  },
                                  Issue {
                                      complete: true,
                                      content: "done".to_string(),
                                      source: SourceLink {
                                          id: "6".to_string(),
                                          url: "http://6".to_string(),
                                      },
                                  },
                              ],
                          });

        doc.report.insert("PackageB".to_string(),
                          Section {
                              header: Header {
                                  package: "PackageB".to_string(),
                                  issue_count: 2,
                                  notes: Some("lol".to_string()),
                              },
                              children: vec![
                                  Issue{
                                      complete: false,
                                      content: "not done".to_string(),
                                      source: SourceLink {
                                          id: "7".to_string(),
                                          url: "http://7".to_string(),
                                      },

                                  },
                                  Issue {
                                      complete: true,
                                      content: "done".to_string(),
                                      source: SourceLink {
                                          id: "8".to_string(),
                                          url: "http://8".to_string(),
                                      },
                                  },
                              ],
                          });


        assert_eq!(doc.count_issues(), 4)
    }
}
