use std::{net::SocketAddr, sync::Arc};

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use ear_api::{
    AncLevel, ApiState, BatteryStatus, CustomEq, EarManager, EarSide, EnhancedBassState, EqMode,
    SerialIdentity, SessionInfo, serve_http,
};
use reqwest::{Client, Method};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

#[derive(Parser)]
#[command(
    name = "ear-api",
    version,
    about = "Control Nothing Ear devices from the CLI or via HTTP"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        default_value = "http://127.0.0.1:8787",
        help = "HTTP endpoint for the running API server"
    )]
    endpoint: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server(ServerOpts),
    Connect(ConnectArgs),
    AutoConnect(AutoConnectArgs),
    Disconnect,
    Session,
    Detect,
    Battery,
    Anc {
        #[command(subcommand)]
        action: AncCommand,
    },
    Eq {
        #[command(subcommand)]
        action: EqCommand,
    },
    CustomEq {
        #[command(subcommand)]
        action: CustomEqCommand,
    },
    Latency {
        #[command(subcommand)]
        action: SwitchCommand,
    },
    InEar {
        #[command(subcommand)]
        action: SwitchCommand,
    },
    EnhancedBass {
        #[command(subcommand)]
        action: EnhancedBassCommand,
    },
    PersonalizedAnc {
        #[command(subcommand)]
        action: SwitchCommand,
    },
    Ring(RingArgs),
}

#[derive(Parser)]
struct ServerOpts {
    #[arg(long, default_value = "127.0.0.1:8787")]
    addr: String,
}

#[derive(Parser)]
struct ConnectArgs {
    #[arg(long, help = "Bluetooth device address (e.g., 00:11:22:33:44:55)")]
    address: String,
    #[arg(long, default_value = "1", help = "RFCOMM channel (default: 1)")]
    channel: u8,
    #[arg(long)]
    model_id: Option<String>,
    #[arg(long)]
    sku: Option<String>,
    #[arg(long)]
    base: Option<ModelBaseArg>,
}

#[derive(Subcommand)]
enum AncCommand {
    Get,
    Set { level: AncLevel },
}

#[derive(Subcommand)]
enum EqCommand {
    Get,
    Set { mode: u8 },
}

#[derive(Subcommand)]
enum CustomEqCommand {
    Get,
    Set {
        #[arg(long)]
        bass: f32,
        #[arg(long)]
        mid: f32,
        #[arg(long)]
        treble: f32,
    },
}

#[derive(Subcommand)]
enum SwitchCommand {
    Get,
    Set {
        #[arg(long)]
        enabled: bool,
    },
}

#[derive(Subcommand)]
enum EnhancedBassCommand {
    Get,
    Set {
        #[arg(long)]
        enabled: bool,
        #[arg(long, default_value = "0")]
        level: u8,
    },
}

#[derive(Parser)]
struct RingArgs {
    #[arg(long)]
    enable: bool,
    #[arg(long)]
    side: Option<EarSide>,
}

#[derive(Parser)]
struct AutoConnectArgs {
    #[arg(long)]
    bluetooth_address: Option<String>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    rfcomm: Option<String>,
    #[arg(long)]
    channel: Option<u8>,
    #[arg(long)]
    baud_rate: Option<u32>,
    #[arg(long)]
    sku: Option<String>,
}

#[derive(Clone)]
struct ApiClient {
    client: Client,
    base: String,
}

impl ApiClient {
    fn new(base: String) -> Self {
        Self {
            client: Client::new(),
            base,
        }
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(Method::GET, path, Option::<Value>::None).await
    }

    async fn post<T, B>(&self, path: &str, body: B) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request(Method::POST, path, Some(body)).await
    }

    async fn delete<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(Method::DELETE, path, Option::<Value>::None)
            .await
    }

    async fn request<T, B>(&self, method: Method, path: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = self.url(path);
        let mut req = self.client.request(method, url);
        if let Some(payload) = body {
            req = req.json(&payload);
        }
        let resp = req.send().await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let status = resp.status();
            let text = resp.text().await?;
            Err(anyhow!("request failed ({status}): {text}"))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ConnectRequest {
    address: String,
    #[serde(default = "default_rfcomm_channel")]
    channel: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<ModelSelector>,
}

#[derive(Debug, Clone, Serialize)]
struct AutoConnectRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sku: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ModelSelector {
    #[serde(skip_serializing_if = "Option::is_none")]
    model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sku: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base: Option<String>,
}

#[derive(Clone)]
struct ModelBaseArg(String);

impl std::str::FromStr for ModelBaseArg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.trim().to_uppercase()))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Server(opts) => run_server(opts).await,
        _ => run_client(cli).await,
    }
}

async fn run_server(opts: ServerOpts) -> Result<()> {
    tracing_subscriber::fmt::init();
    let manager = Arc::new(EarManager::new());
    let addr: SocketAddr = opts.addr.parse()?;
    let state = ApiState { manager };
    serve_http(state, addr).await?;
    Ok(())
}

async fn run_client(cli: Cli) -> Result<()> {
    let client = ApiClient::new(cli.endpoint);
    match cli.command {
        Commands::Server(_) => unreachable!(),
        Commands::Connect(args) => {
            let selector = build_selector(&args);
            let req = ConnectRequest {
                address: args.address,
                channel: args.channel,
                model: selector,
            };
            let resp: SessionInfo = client.post("/api/session/connect", req).await?;
            print_json(&resp)?;
        }
        Commands::AutoConnect(args) => {
            let body = AutoConnectRequestBody {
                address: args.bluetooth_address.clone(),
                name: args.name.clone(),
                channel: args.channel,
                sku: args.sku.clone(),
            };
            let resp: SessionInfo = client.post("/api/session/auto-connect", body).await?;
            print_json(&resp)?;
        }
        Commands::Disconnect => {
            let resp: Value = client.delete("/api/session").await?;
            print_json(&resp)?;
        }
        Commands::Session => {
            let info: SessionInfo = client.get("/api/session").await?;
            print_json(&info)?;
        }
        Commands::Detect => {
            let resp: SerialIdentity = client
                .post("/api/session/detect", serde_json::json!({}))
                .await?;
            print_json(&resp)?;
        }
        Commands::Battery => {
            let battery: BatteryStatus = client.get("/api/battery").await?;
            print_json(&battery)?;
        }
        Commands::Anc { action } => match action {
            AncCommand::Get => {
                let anc: AncLevel = client.get("/api/anc").await?;
                print_json(&anc)?;
            }
            AncCommand::Set { level } => {
                let body = serde_json::json!({ "level": level });
                let resp: Value = client.post("/api/anc", body).await?;
                print_json(&resp)?;
            }
        },
        Commands::Eq { action } => match action {
            EqCommand::Get => {
                let eq: EqMode = client.get("/api/eq").await?;
                print_json(&eq)?;
            }
            EqCommand::Set { mode } => {
                let body = serde_json::json!({ "mode": mode });
                let resp: Value = client.post("/api/eq", body).await?;
                print_json(&resp)?;
            }
        },
        Commands::CustomEq { action } => match action {
            CustomEqCommand::Get => {
                let eq: CustomEq = client.get("/api/eq/custom").await?;
                print_json(&eq)?;
            }
            CustomEqCommand::Set { bass, mid, treble } => {
                let body = CustomEq { bass, mid, treble };
                let resp: Value = client.post("/api/eq/custom", body).await?;
                print_json(&resp)?;
            }
        },
        Commands::Latency { action } => {
            handle_switch_command(&client, "/api/latency", action).await?;
        }
        Commands::InEar { action } => {
            handle_switch_command(&client, "/api/in-ear", action).await?;
        }
        Commands::EnhancedBass { action } => match action {
            EnhancedBassCommand::Get => {
                let resp: EnhancedBassState = client.get("/api/enhanced-bass").await?;
                print_json(&resp)?;
            }
            EnhancedBassCommand::Set { enabled, level } => {
                let body = EnhancedBassState { enabled, level };
                let resp: Value = client.post("/api/enhanced-bass", body).await?;
                print_json(&resp)?;
            }
        },
        Commands::PersonalizedAnc { action } => {
            handle_switch_command(&client, "/api/personalized-anc", action).await?;
        }
        Commands::Ring(args) => {
            let body = serde_json::json!({
                "enable": args.enable,
                "side": args.side
            });
            let resp: Value = client.post("/api/ring", body).await?;
            print_json(&resp)?;
        }
    }
    Ok(())
}

async fn handle_switch_command(
    client: &ApiClient,
    path: &str,
    action: SwitchCommand,
) -> Result<()> {
    match action {
        SwitchCommand::Get => {
            let resp: Value = client.get(path).await?;
            print_json(&resp)?;
        }
        SwitchCommand::Set { enabled } => {
            let body = serde_json::json!({ "enabled": enabled });
            let resp: Value = client.post(path, body).await?;
            print_json(&resp)?;
        }
    }
    Ok(())
}

fn build_selector(args: &ConnectArgs) -> Option<ModelSelector> {
    if args.model_id.is_none() && args.sku.is_none() && args.base.is_none() {
        return None;
    }
    Some(ModelSelector {
        model_id: args.model_id.clone(),
        sku: args.sku.clone(),
        base: args.base.as_ref().map(|b| b.0.clone()),
    })
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
