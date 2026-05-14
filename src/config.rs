use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum Mode {
    Triangle = 1 << 0,
    Pump = 1 << 1,
    Dump = 1 << 2,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Huita Detector")]
struct Args {
    #[arg(short, long, value_delimiter = ',')]
    pairs: Vec<String>,

    #[arg(short, long, value_enum, default_value_t = Mode::Triangle)]
    mode: Mode,

    #[arg(short, long, default_value_t = -1.0)]
    threshold: f64,
    
    #[arg(long, default_value_t = -1.0)]
    target_price: f64,
    
	#[arg(long, default_value_t = String::from("USDT"))]
    initial_asset: String,
     
    #[arg(long, default_value_t = 3000)]
    port: u16,
    
}

#[derive(Debug)]
pub struct Config {
    pub pairs: Vec<String>,
    pub mode: Mode,
    pub threshold: f64, 
    pub target_price: f64,
    pub initial_asset: String,
    pub port: u16,

}

impl Config {
    pub fn load() -> Self {
        let args = Args::parse();
		dotenvy::dotenv().ok();
        let pairs = if args.pairs.is_empty() {
            vec![
                "BTCUSDT".to_string(),
                "ETHBTC".to_string(),
                "ETHUSDT".to_string(),
            ]
        } else {
            args.pairs
                .into_iter()
                .map(|s| s.to_uppercase())
                .collect()
        };

        Self {
            pairs,
            mode: args.mode,
            threshold: args.threshold,
            target_price: args.target_price,
            initial_asset: args.initial_asset.to_uppercase(),
            port: args.port
        }
    }
}
