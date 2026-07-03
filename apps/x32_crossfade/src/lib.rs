use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Smoothly interpolate faders, EQs, and dynamics parameters between two scenes", long_about = None)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Path to the first scene file (.scn)
    #[arg(short = 'a', long)]
    pub scene_a: String,

    /// Path to the second scene file (.scn)
    #[arg(short = 'b', long)]
    pub scene_b: String,

    /// Duration of the crossfade in seconds
    #[arg(short, long, default_value_t = 5.0)]
    pub duration: f64,
}

pub async fn run(args: Args) -> Result<()> {
    Ok(())
}
