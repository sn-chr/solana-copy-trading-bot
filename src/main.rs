use anyhow::Result;
use clap::{Parser, Subcommand};
use chrono::Utc;
use std::{env, fs, path::Path};

#[derive(Parser, Debug)]
#[command(name = "SolGod", version, about = "Solana Copy Trading Bot - CLI")]
struct Cli {
    /// Simulation without sending transactions
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Copy-trade wallets
    Copy {
        /// List of wallets (comma-separated)
        #[arg(long)]
        wallets: String,
        /// Minimum winrate (%)
        #[arg(long, default_value_t = 58)]
        min_winrate: u32,
        /// Maximum holding time (hours)
        #[arg(long, default_value_t = 72)]
        max_hold_h: u32,
    },
    /// Simulate trading volume
    Volume {
        /// Token mint address
        #[arg(long)]
        token: String,
        /// Transactions per second
        #[arg(long, default_value_t = 0.4)]
        tps: f32,
        /// Jitter factor
        #[arg(long, default_value_t = 0.2)]
        jitter: f32,
        /// Burst size
        #[arg(long, default_value_t = 3)]
        burst: u32,
        /// Cooldown in seconds (format: a-b)
        #[arg(long, default_value = "25-60")]
        cooldown: String,
    },
    /// Snipe newly deployed tokens
    Snipe {
        /// Sources (comma-separated)
        #[arg(long, default_value = "pumpfun,moonshot")]
        sources: String,
        /// Max market cap
        #[arg(long, default_value = "250k")]
        max_mcap: String,
        /// Anti-rug protection
        #[arg(long, default_value_t = true)]
        anti_rug: bool,
    },
    /// Deploy token and seed LP
    Launch {
        #[arg(long)]
        name: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        supply: String,
        #[arg(long)]
        lp: String,
        #[arg(long)]
        lock: String,
    },
}

struct EnvCfg {
    primary_rpc: String,
    fallbacks: Vec<String>,
    export_excel: bool,
    export_dir: String,
    slippage_bps: u32,
}

fn load_env() -> EnvCfg {
    let _ = dotenvy::dotenv();
    EnvCfg {
        primary_rpc: env::var("PRIMARY_RPC").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into()),
        fallbacks: env::var("FALLBACK_RPCS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect(),
        export_excel: env::var("EXPORT_EXCEL").unwrap_or_else(|_| "false".into()) == "true",
        export_dir: env::var("EXPORT_DIR").unwrap_or_else(|_| "./exports".into()),
        slippage_bps: env::var("SLIPPAGE_BPS").ok().and_then(|v| v.parse().ok()).unwrap_or(150),
    }
}

fn maybe_export_csv(dir: &str, name: &str, headers: &[&str], rows: &[Vec<String>]) -> Result<()> {
    fs::create_dir_all(dir)?;
    let path = Path::new(dir).join(format!("{name}.csv"));
    let mut csv = String::new();
    csv.push_str(&headers.join(","));
    csv.push('\n');
    for r in rows {
        csv.push_str(&r.join(","));
        csv.push('\n');
    }
    fs::write(&path, csv)?;
    println!("Exported: {}", path.display());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = load_env();

    println!("SolGod | RPC: {}", cfg.primary_rpc);
    if !cfg.fallbacks.is_empty() {
        println!("Fallback RPCs: {}", cfg.fallbacks.join(", "));
    }
    println!("Slippage: {} bps | Export: {} -> {}", cfg.slippage_bps, cfg.export_excel, cfg.export_dir);
    if cli.dry_run { println!("Mode: DRY-RUN"); }

    match cli.cmd {
        Commands::Copy { wallets, min_winrate, max_hold_h } => {
            let when = Utc::now();
            println!("COPY | wallets={wallets} | min_winrate={min_winrate}% | max_hold_h={max_hold_h} | {}", when);
            if cfg.export_excel {
                maybe_export_csv(
                    &cfg.export_dir,
                    "copy_summary",
                    &["wallet","winrate","roi","trades","avg_hold_h"],
                    &[
                        vec!["addr1".into(),"62".into(),"18.4".into(),"112".into(),"36".into()],
                        vec!["addr2".into(),"59".into(),"12.1".into(),"87".into(),"28".into()],
                    ],
                )?;
            }
        }
        Commands::Volume { token, tps, jitter, burst, cooldown } => {
            println!("VOLUME | token={token} | tps={tps} | jitter={jitter} | burst={burst} | cooldown={cooldown}");
            if cli.dry_run {
                println!("(dry-run) Transactions would be generated with human-like timing...");
            } else {
                println!("(mock) Sending volume transactions...");
            }
        }
        Commands::Snipe { sources, max_mcap, anti_rug } => {
            println!("SNIPE | sources={sources} | max_mcap={max_mcap} | anti_rug={anti_rug}");
            if cli.dry_run {
                println!("(dry-run) Monitoring new listings and simulating entry...");
            } else {
                println!("(mock) Subscribing to listings and executing fast entry...");
            }
        }
        Commands::Launch { name, symbol, supply, lp, lock } => {
            println!("LAUNCH | {name} ({symbol}) | supply={supply} | lp={lp} | lock={lock}");
            if cli.dry_run {
                println!("(dry-run) Simulating token deployment and LP creation...");
            } else {
                println!("(mock) Deploying token and preparing liquidity pool...");
            }
        }
    }

    Ok(())
}
