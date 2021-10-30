use futures::executor::block_on;
use reqwest::Url;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::clap;
use structopt::StructOpt;

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
    url: Option<Url>,
}

fn main() {
    let opts = Opts::from_args();

    let results: Vec<String> = if let Some(url) = opts.url {
        let thing = http::resolve_url(url.into());

        let t = block_on(thing);

        println!("{:?}", t);

        vec![]
    } else {
        vec![]
    };
}
