use clap::Parser;
use tide_disco::Url;

#[derive(Parser, Clone, Debug)]
pub struct Options {
    #[clap(long, env = "HOTSHOT_EVENTS_API_URL")]
    pub events_url: Url,
}
