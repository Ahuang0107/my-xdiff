use clap::Parser;

use my_xdiff::cli::{Action, Args, RunArgs};
use my_xdiff::config::DiffConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    // println!("{:?}", args);
    match args.action {
        Action::Run(args) => run(args).await?,
        _ => panic!("Not implemented"),
    }
    Ok(())
}

async fn run(args: RunArgs) -> anyhow::Result<()> {
    let config_file = args.config.unwrap_or_else(|| "./config.yml".to_string());
    let config = DiffConfig::load_yaml(&config_file).await?;
    let profile = config.get_profile(&args.profile).ok_or_else(|| {
        anyhow::anyhow!(
            "Profile {} not found in config file {}",
            args.profile,
            config_file
        )
    })?;
    let extra_args = args.extra_params.into();
    profile.diff(extra_args).await?;
    Ok(())
}
