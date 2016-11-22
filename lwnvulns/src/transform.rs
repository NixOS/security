use std::collections::HashMap;
use parse::{Document, Section};
use tokenize::Header;

pub fn partition_completed(src: Document) -> (Document, Document) {
    let mut todo = Document {
        preamble: src.preamble,
        report: HashMap::new(),
    };
    let mut done = Document {
        preamble: vec![],
        report: HashMap::new(),
    };

    for (package, section) in src.report {
        let mut header = section.header.clone();
        header.issue_count = 0;

        for issue in section.children {
            let ref mut doc = if issue.complete { &mut done } else { &mut todo };
            let mut section = doc.report.entry(package.clone()).or_insert(Section {
                header: header.clone(),
                children: vec![],
            });

            section.children.push(issue);
            section.header.issue_count += 1;
        }
    }

    return (todo, done);
}


pub fn collapse_anemic(mut src: Document) -> Document {
    let mut collapsed = Document {
        preamble: src.preamble,
        report: HashMap::new(),
    };

    let mut reports: Vec<String> = src.report.keys().map(|key| key.clone()).collect();
    reports.sort();
    for key in reports {
        let section = src.report.remove(&key).unwrap();

        if section.children.len() > 1 {
            collapsed.report.insert(section.header.package.to_string().clone(), section);
        } else {
            let mut destination =
                collapsed.report.entry("Assorted".to_string()).or_insert(Section {
                    header: Header {
                        package: "Assorted".to_string(),
                        issue_count: 0,
                        notes: None,
                    },
                    children: vec![],
                });

            for issue in section.children {
                destination.children.push(issue);
                destination.header.issue_count += 1;
            }
        }


    }

    return collapsed;
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use parse::{Document, Section};
    use tokenize::{Token, Header, Issue, SourceLink};
    use super::*;

    #[test]
    fn test_lwnvulns_partition() {
        let mut input = Document {
            preamble: vec![
                Token::Preamble("this".to_string()),
                Token::Preamble("is".to_string()),
                Token::Preamble("garbage".to_string()),
                Token::Preamble("### Here too! ( :) )".to_string()),
            ],
            report: HashMap::new(),
        };

        input.report.insert("This is a header too".to_string(),
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
                                     },
                                 ],
                            });

        let mut expect_todo = Document {
            preamble: vec![
                Token::Preamble("this".to_string()),
                Token::Preamble("is".to_string()),
                Token::Preamble("garbage".to_string()),
                Token::Preamble("### Here too! ( :) )".to_string()),
            ],
            report: HashMap::new(),
        };

        expect_todo.report.insert("This is a header too".to_string(),
                                  Section {
                                      header: Header {
                                          package: "This is a header too".to_string(),
                                          issue_count: 1,
                                          notes: Some(" lol".to_string()),
                                      },
                                      children: vec![
                                          Issue {
                                              complete: false,
                                              content: "not done".to_string(),
                                              source: SourceLink {
                                                  id: "not done".to_string(),
                                                  url: "http://not done".to_string(),
                                              },
                                          },
                                      ],
                                  });

        let mut expect_done = Document {
            preamble: vec![],
            report: HashMap::new(),
        };

        expect_done.report.insert("This is a header too".to_string(),
                                  Section {
                                      header: Header {
                                          package: "This is a header too".to_string(),
                                          issue_count: 1,
                                          notes: Some(" lol".to_string()),
                                      },
                                      children: vec![
                                          Issue {
                                              complete: true,
                                              content: "done".to_string(),
                                              source: SourceLink {
                                                  id: "done".to_string(),
                                                  url: "http://done".to_string(),
                                              },
                                          },
                                      ],
                                  });



        let (todo, done) = partition_completed(input);
        assert_eq!(todo, expect_todo);
        assert_eq!(done, expect_done);

    }

    #[test]
    fn test_collapse_anemic() {
        let mut input = Document {
            preamble: vec![
                Token::Preamble("this".to_string()),
                Token::Preamble("is".to_string()),
                Token::Preamble("garbage".to_string()),
                Token::Preamble("### Here too! ( :) )".to_string()),
            ],
            report: HashMap::new(),
        };

        input.report.insert("This is a header".to_string(),
                            Section {
                                header: Header {
                                    package: "This is a header".to_string(),
                                    issue_count: 1,
                                    notes: Some(" lol".to_string()),
                                },
                                children: vec![
                                     Issue {
                                         complete: false,
                                         content: "not done".to_string(),
                                         source: SourceLink {
                                             id: "not done".to_string(),
                                             url: "http://not done".to_string(),
                                         },
                                     },
                                 ],
                            });

        input.report.insert("This is a header too".to_string(),
                            Section {
                                header: Header {
                                    package: "This is a header too".to_string(),
                                    issue_count: 1,
                                    notes: Some(" lol".to_string()),
                                },
                                children: vec![
                                     Issue {
                                         complete: true,
                                         content: "done".to_string(),
                                         source: SourceLink {
                                             id: "done".to_string(),
                                             url: "http://done".to_string(),
                                         },
                                     },
                                 ],
                            });


        let mut expect = Document {
            preamble: vec![
                Token::Preamble("this".to_string()),
                Token::Preamble("is".to_string()),
                Token::Preamble("garbage".to_string()),
                Token::Preamble("### Here too! ( :) )".to_string()),
            ],
            report: HashMap::new(),
        };

        expect.report.insert("Assorted".to_string(),
                             Section {
                                 header: Header {
                                     package: "Assorted".to_string(),
                                     issue_count: 2,
                                     notes: None,
                                 },
                                 children: vec![
                                          Issue {
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
                                          },
                                      ],
                             });


        let collapsed = collapse_anemic(input);
        assert_eq!(expect, collapsed);

    }

}
