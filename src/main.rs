use clap::Parser;


#[derive(Parser)]
#[command(name = "music-text")]
#[command(about = "A hand-written recursive descent music-text parser")]
struct Cli {
    /// Start web server mode
    #[arg(long)]
    web: bool,
}


#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Web server mode
    if cli.web {
        music_text::web::start_server().await?;
        return Ok(());
    }

    Ok(())
}




