use clap::{Parser, ValueEnum};

mod extbus;
mod intio;
mod iobus;
mod ledmatrix;
mod utils;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
enum Module {
    All,
    Extbus,
    Intio,
    Iobus,
    Ledmatrix
}

#[derive(Parser, Debug)]
struct Mode {
    /// Specify module for test
    #[arg(value_enum, default_value_t = Module::All)]
    module: Module
}

fn main() {
    let config = utils::parse_config();
    let mode = Mode::parse();
    match mode.module {
        Module::All => {
            intio::test(&config.intio);
        },
        Module::Extbus => (),
        Module::Intio => intio::test(&config.intio),
        Module::Iobus => (),
        Module::Ledmatrix => ()
    }
}
