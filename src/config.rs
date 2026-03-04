use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Runs an ettac script
pub struct Config {
    #[argh(positional)]
    /// host(s) to deploy to
    pub hosts: Vec<String>,

    #[argh(option, short = 's', default = "String::from(\"deploy.lua\")")]
    /// path of the script to run
    pub script: String,
}
