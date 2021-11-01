use reqwest::Url;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::clap;
use structopt::StructOpt;
use tabular::Row;
use tabular::Table;
use tokio::runtime::Runtime;

mod http;

#[derive(StructOpt, Debug)]
enum OutputFormat {
    JSON,
    Text,
}

impl FromStr for OutputFormat {
    type Err = clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(OutputFormat::JSON),
            "text" => Ok(OutputFormat::Text),
            _ => Err(Self::Err::argument_not_found_auto(s)),
        }
    }
}

#[derive(StructOpt, Debug)]
struct Opts {
    /// newline delimited list of urls to test
    #[structopt(short, long, name = "FILE")]
    input: Option<PathBuf>,

    /// can be either "json" or "text", defaults to "text"
    #[structopt(short, long, default_value = "text")]
    output: OutputFormat,

    /// can use if just checking a single url
    #[structopt(short, long)]
    url: Option<Vec<Url>>,
}

fn main() {
    let opts = Opts::from_args();

    let hosts: Vec<Url> = if let Some(urls) = opts.url {
        urls
    } else if let Some(input) = opts.input {
        read_hosts_from_file(input).unwrap()
    } else {
        panic!("yikes");
    };

    let processor = http::Processor::from_hosts(hosts);
    let runtime = Runtime::new().unwrap();
    let results = runtime.block_on(processor.process());

    if let Ok(urls) = results {
        let output = match opts.output {
            OutputFormat::JSON => {
                let urls = urls
                    .iter()
                    .map(|(src, dest)| {
                        HashMap::<String, String>::from_iter(
                            [
                                ("source".to_string(), src.to_string()),
                                ("redirect".to_string(), dest.to_string()),
                            ]
                            .into_iter(),
                        )
                    })
                    .collect::<Vec<_>>();

                serde_json::to_string(&urls).unwrap()
            }

            OutputFormat::Text => urls
                .iter()
                .fold(Table::new("{:<}    {:<}"), |mut table, (src, dest)| {
                    table.add_row(Row::new().with_cell(src).with_cell(dest));
                    table
                })
                .to_string(),
        };

        println!("{}", output)
    } else {
        println!("Failed to fetch urls: {:?}", results)
    }
}

fn read_hosts_from_file(path: PathBuf) -> Result<Vec<Url>, std::io::Error> {
    let file = File::open(path)?;
    let buf = BufReader::new(file).lines();

    buf.filter(|l| l.is_ok())
        .map(|l| {
            Url::from_str(&l.unwrap())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
        })
        .collect()
}
