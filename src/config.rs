use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Huita Detector")]
struct Args {
    #[arg(short, long, value_delimiter = ',')]
    pairs: Option<Vec<String>>,

   // #[arg(short, long)]
   // best_choice: usize
}

#[derive(Debug)]
pub struct Config {
    pub pairs: [&'static str; 3],
   // pub best_choice: usize
}

impl Config {
    pub fn load() -> Self {
        let args = Args::parse();
		
        let default_pairs = ["ETHBTC", "BNBETH", "ETHUSD"];

        let final_pairs = match args.pairs {
            Some(p) if p.len() == 3 => [
                Self::leak_str(p[0].to_uppercase()),
                Self::leak_str(p[1].to_uppercase()),
                Self::leak_str(p[2].to_uppercase()),
            ],
            _ => default_pairs,
        };

        Self { pairs: final_pairs, } //best_choice: args.best_choice }
    }

    fn leak_str(s: String) -> &'static str {
        Box::leak(s.into_boxed_str())
    }
}
