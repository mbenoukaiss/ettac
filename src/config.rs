use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Runs an ettac script
pub struct Config {
    #[argh(positional)]
    /// path to the script to run
    pub script: String,

    #[argh(option, short = 'h')]
    /// host(s) to deploy to
    pub host: Vec<String>,
}
