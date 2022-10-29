use clap::{Parser, ValueEnum};

mod extbus;
mod intio;
mod iobus;
mod ledmatrix;
mod ledpanel;
mod ccnet;
mod ccnet_dev;
mod cctalk_dev;
mod wiegand;
mod wiegand_dev;
mod terminal;
mod utils;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
enum Module {
    All,
    Extbus,
    Intio,
    Iobus,
    Ledmatrix,
    Ledpanel,
    Ccnet,
    Cctalk,
    Terminal,
    Rfid
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

fn print_test<T>(name: &str, config: &T, func: fn(&T) -> Result<(), String>) -> Result<(), ()> {
    match func(config) {
        Ok(_) => {
            println!("Module '{}' tested ok", name);
            Ok(())
        },
        Err(e) => {
            println!("Module '{}' tested fail: {}", name, e);
            Err(())
        }
    }
}

fn main() -> Result<(), ()> {
    let mode = Mode::parse();
    let config = utils::parse_config(&mode.config);
    match mode.module {
        Module::All => {
            print_test("Internal IO", &config.intio, intio::test)?;
            print_test("IO Bus", &config.iobus, iobus::test)?;
            print_test("Ledmatrix", &config.ledmatrix, ledmatrix::test)?;
            print_test("Ledpanel", &config.ledpanel, ledpanel::test)?;
            print_test("Rfid", &config.rfid, wiegand_dev::test)?;
            print_test("Ccnet", &config.ccnet, ccnet_dev::test)?;
            print_test("Cctalk", &config.cctalk, cctalk_dev::test)?;
            print_test("Terminal", &config.terminal, terminal::test)?;
            Ok(())
        },
        Module::Extbus => Err(()),
        Module::Intio => print_test("Internal IO", &config.intio, intio::test),
        Module::Iobus => print_test("IO Bus", &config.iobus, iobus::test),
        Module::Ledmatrix => print_test("Ledmatrix", &config.ledmatrix, ledmatrix::test),
        Module::Ledpanel => print_test("Ledpanel", &config.ledpanel, ledpanel::test),
        Module::Rfid => print_test("Rfid", &config.rfid, wiegand_dev::test),
        Module::Ccnet => print_test("Ccnet", &config.ccnet, ccnet_dev::test),
        Module::Cctalk => print_test("Cctalk", &config.cctalk, cctalk_dev::test),
        Module::Terminal => print_test("Terminal", &config.terminal, terminal::test)
    }
}
