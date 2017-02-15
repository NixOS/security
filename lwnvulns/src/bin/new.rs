extern crate lwnvulns;
extern crate scraper;
extern crate curl;

use lwnvulns::tokenize::{tokenize, Token, Header, Issue, SourceLink};
use lwnvulns::parse::parse;
use lwnvulns::writer::write;
use lwnvulns::transform::collapse_anemic;
use scraper::{Html, Selector};
use curl::easy::Easy;

use std::io::{Read, Write};
use std::fs::File;

struct LwnRow {
    url: String,
    id: String,
    packages: String,
    description: String,
}

fn load_db(src: &str) -> Vec<String> {
    let mut db: Vec<String> = vec![];
    {
        let mut f = File::open(src).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        for entry in s.split_whitespace() {
            db.push(entry.to_string());
        }
    }

    return db;
}

fn main() {
    let db = load_db("./db");


    let mut tokens: Vec<Token> = tokenize(include_str!("./instructions.md").to_string());


    let mut page = 0;
    let mut pages_with_nothing = 0;

    loop {
        let mut found = 0;
        for token in tokens_from_html(fetch_page(page)) {
            if let Token::Issue(ref issue) = token {
                if db.contains(&issue.source.id) {
                    continue;
                } else {
                    found += 1;
                }
            }
            tokens.push(token);
        }

        writeln!(&mut std::io::stderr(),
                 "Page {page} yielded {found} issues, after {none} pages with nothing",
                 page = page,
                 found = found,
                 none = pages_with_nothing)
            .unwrap();

        page += 1;

        if found == 0 {
            pages_with_nothing += 1;
        } else {
            pages_with_nothing = 0;
        }

        if pages_with_nothing >= 2 {
            break;
        }

        if page > 10 {
            panic!("Not sure we should be hitting 10 pages!");
        }
    }

    print!("{}", write(&collapse_anemic(parse(tokens).unwrap())));

}

fn fetch_page(page: usize) -> scraper::Html {
    let url = format!("https://lwn.net/Vulnerabilities/?n={}&offset={}",
                      100,
                      page * 100);
    let mut data = Vec::new();

    let mut easy = Easy::new();
    easy.url(&url).unwrap();

    {
        let mut transfer = easy.transfer();
        transfer.write_function(|chunk| {
                data.extend_from_slice(chunk);
                Ok(chunk.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    return Html::parse_document(&String::from_utf8(data).unwrap());
}

fn tokens_from_html(html: scraper::Html) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let rows = Selector::parse("tr").unwrap();
    let links = Selector::parse("td:nth-child(1) a").unwrap();
    let packages = Selector::parse("td:nth-child(3)").unwrap();
    let descriptions = Selector::parse("td:nth-child(4)").unwrap();
    for row in html.select(&rows) {
        if let Some(link) = row.select(&links).next() {
            let package = row.select(&packages).next().unwrap().text().next().unwrap();
            let description = row.select(&descriptions).next().unwrap().text().next().unwrap();

            let lwnrow = LwnRow {
                url: link.value().attr("href").unwrap().to_string(),
                id: link.inner_html(),
                packages: package.to_string(),
                description: description.to_string(),
            };
            for token in row_to_tokens(lwnrow) {
                tokens.push(token);
            }
        }
    }

    return tokens;
}

fn row_to_tokens(row: LwnRow) -> Vec<Token> {
    let mut tokens = vec![];

    let url = format!("https://lwn.net{}", row.url);
    for pkg in split_packages(row.packages) {
        tokens.push(Token::Header(Header {
            issue_count: 0,
            notes: None,
            package: pkg.clone(),
        }));

        tokens.push(Token::Issue(Issue {
            complete: false,
            source: SourceLink {
                id: row.id.clone(),
                url: url.clone(),
            },
            content: format!(
                "([search]({search_url}), [files]({files_url}))  {description}",
                search_url = format!(
                    "http://search.nix.gsc.io/?q={package}&i=fosho&repos=nixos-nixpkgs",
                    package=pkg
                ),
                files_url = format!(
                    "{root}?utf8=%E2%9C%93&q={package}+in%3Apath&type=Code",
                    root="https://github.com/NixOS/nixpkgs/search",
                    package=pkg
                ),
                description=row.description
            ),
        }));
    }

    return tokens;
}

fn split_packages(packages: String) -> Vec<String> {
    let pkgs = packages.split_whitespace()
        .map(|s| s.replace(",", "").to_string())
        .map(|s| if s == "tiff" {
            "libtiff".to_string()
        } else if s == "libgc" {
            "boehm-gc".to_string()
        } else if s == "perl-DBD-MySQL" {
            "DBD-mysql".to_string()
        } else if s == "httpd" {
            "apache-httpd".to_string()
        } else {
            s
        })
        .collect();

    return pkgs;
}
