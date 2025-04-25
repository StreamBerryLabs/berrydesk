use berry_core::control::{Request, Response, socket_path};
use clap::{Parser, Subcommand};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Parser)]
#[command(name = "berry", version, about = "BerryDesk CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        url: String,
        #[arg(short, long)]
        name: Option<String>,
    },
    List,
    Start {
        name_or_id: String,
    },
    Remove {
        name_or_id: String,
    },
    Stop,
    Status,
    Snapshot { out: Option<String> },
    Doctor,
    InstallDriver,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("warn").init();
    let cli = Cli::parse();
    match cli.cmd {
        Commands::Add { url, name } => {
            let name = name.unwrap_or_else(|| url.clone());
            let resp = request(Request::AddSource { name, url }).await?;
            print_resp(&resp);
        }
        Commands::List => {
            let resp = request(Request::ListSources).await?;
            if resp.ok {
                if let Some(data) = resp.data {
                    println!("{}", serde_json::to_string_pretty(&data)?);
                }
            } else {
                eprintln!("{}", resp.message);
            }
        }
        Commands::Start { name_or_id } => {
            let id = resolve_id(&name_or_id).await.unwrap_or(name_or_id);
            let resp = request(Request::Start { id }).await?;
            print_resp(&resp);
        }
        Commands::Remove { name_or_id } => {
            let id = resolve_id(&name_or_id).await.unwrap_or(name_or_id);
            let resp = request(Request::RemoveSource { id }).await?;
            print_resp(&resp);
        }
        Commands::Stop => {
            let resp = request(Request::Stop).await?;
            print_resp(&resp);
        }
        Commands::Status => {
            let resp = request(Request::Status).await?;
            print_resp(&resp);
            if let Some(data) = resp.data {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        }
        Commands::Snapshot { out } => {
            let resp = request(Request::Snapshot).await?;
            print_resp(&resp);
            if resp.ok
                && let Some(data) = resp.data
                && let Some(b64) = data.get("jpeg_base64").and_then(|v| v.as_str())
            {
                        use base64::Engine;
                        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).unwrap_or_default();
                        if let Some(path) = out {
                            std::fs::write(&path, &bytes)?;
                            println!("wrote {} ({} bytes)", path, bytes.len());
                        } else {
                            println!("captured {} bytes", bytes.len());
                        }
            }
        }
        Commands::Doctor => {
            let resp = request(Request::Doctor).await?;
            print_resp(&resp);
            if let Some(data) = resp.data {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        }
        Commands::InstallDriver => {
            let resp = request(Request::InstallDriver).await?;
            print_resp(&resp);
        }
    }
    Ok(())
}

async fn request(req: Request) -> anyhow::Result<Response> {
    // ensure daemon running
    let sock = socket_path();
    if !sock.exists() {
        // try to spawn berryd
        let _ = std::process::Command::new("berryd").spawn();
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    let stream = UnixStream::connect(&sock).await?;
    let (r, mut w) = stream.into_split();
    let mut reader = BufReader::new(r);
    let mut line = String::new();
    let mut out = serde_json::to_string(&req)?;
    out.push('\n');
    w.write_all(out.as_bytes()).await?;
    reader.read_line(&mut line).await?;
    let resp: Response = serde_json::from_str(line.trim())?;
    Ok(resp)
}

async fn resolve_id(name_or_id: &str) -> Option<String> {
    let resp = request(Request::ListSources).await.ok()?;
    let data = resp.data?;
    let arr = data.as_array()?;
    for s in arr {
        if s.get("id").and_then(|v| v.as_str()) == Some(name_or_id) {
            return Some(name_or_id.to_string());
        }
        if s.get("name").and_then(|v| v.as_str()) == Some(name_or_id) {
            return s.get("id").and_then(|v| v.as_str()).map(|x| x.to_string());
        }
    }
    None
}

fn print_resp(resp: &Response) {
    if resp.ok {
        println!("OK: {}", resp.message);
    } else {
        eprintln!("ERR: {}", resp.message);
    }
}
