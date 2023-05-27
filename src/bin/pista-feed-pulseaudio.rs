use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[clap(long = "prefix", default_value = "v ")]
    prefix: String,

    #[clap(long = "symbol-mic-on", default_value = "!")]
    symbol_mic_on: String,

    #[clap(long = "symbol-mic-off", default_value = " ")]
    symbol_mic_off: String,
}

impl Cli {
    fn symbols(&self) -> pista_feeds::pulseaudio::Symbols {
        pista_feeds::pulseaudio::Symbols {
            prefix: &self.prefix,
            mic_on: &self.symbol_mic_on,
            mic_off: &self.symbol_mic_off,
            mute: "  X  ",
            equal: "=",
            approx: "~",
        }
    }
}

fn main() -> anyhow::Result<()> {
    pista_feeds::log::init()?;
    let cli = Cli::parse();
    pista_feeds::pulseaudio::run(cli.symbols())
}
