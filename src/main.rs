use clap::{Parser, ValueEnum};
use intio::IntioConfig;
use iobus::IobusConfig;

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
    #[arg(short, long, value_enum, default_value_t = Module::All)]
    module: Module,
    
    /// Config path
    #[arg(short, long, default_value_t = String::from("./config.toml"))]
    config: String
}

fn test_intio(config: &IntioConfig) -> Result<(), ()> {
    match intio::test(config) {
        Ok(descr) => {
            println!("Module 'InternalIO' tested ok: {}", descr);
            Ok(())
        },
        Err(e) => {
            println!("Module 'InternalIO' tested fail: {}", e);
            Err(())
        }
    }
}

fn test_iobus(config: &IobusConfig) -> Result<(), ()> {
    match iobus::test(config) {
        Ok(descr) => {
            println!("Module 'IO Bus' tested ok: {}", descr);
            Ok(())
        },
        Err(e) => {
            println!("Module 'IO Bus' tested fail: {}", e);
            Err(())
        }
    }
}

fn main() -> Result<(), ()> {
    let mode = Mode::parse();
    let config = utils::parse_config(&mode.config);
    match mode.module {
        Module::All => {
            test_intio(&config.intio)?;
            test_iobus(&config.iobus)?;
            Ok(())
        },
        Module::Extbus => Err(()),
        Module::Intio => test_intio(&config.intio),
        Module::Iobus => test_iobus(&config.iobus),
        Module::Ledmatrix => Err(())
    }
}
