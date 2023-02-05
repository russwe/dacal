use std::error::Error;

use dacal:: Dacal;
use clap::{ Parser, Subcommand };

#[derive(Parser)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Open {
        spindle_id: u16,
        disc_number: u8,
    },
    Close {
        spindle_id: u16,
    },
    Status {
        spindle_id: u16,
    },
    List {
        #[arg(short, long)]
        identify: bool,

        #[arg(short, long)]
        status: bool,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Open { spindle_id, disc_number } => cmd_open(*spindle_id, *disc_number),
        Commands::Close { spindle_id } => cmd_close(*spindle_id),
        Commands::Status { spindle_id } => cmd_status(*spindle_id),
        Commands::List { identify, status } => cmd_list(*identify, *status),

    }
}

fn cmd_open(spindle_id: u16, disc_number: u8) -> Result<(), Box<dyn Error>> {
    let d = Dacal::from_id(spindle_id)?;
    d.access_slot(disc_number)?;

    Ok(())
}

fn cmd_close(spindle_id: u16) -> Result<(), Box<dyn Error>> {
    let d = Dacal::from_id(spindle_id)?;
    d.retract_arm()?;

    Ok(())
}

fn cmd_status(spindle_id: u16) -> Result<(), Box<dyn Error>> {
    let d = Dacal::from_id(spindle_id)?;
    let status = d.get_status();

    println!("{}: {:?}", spindle_id, status);

    Ok(())
}

fn cmd_list(identify: bool, status: bool) -> Result<(), Box<dyn Error>> {
    let mut devices = dacal::devices()?;
    devices.sort_by_key(|d| d.id);

    let mut index = 0;
    for d in devices {
        index += 1;
        if identify {
            print!("{:03}: ", index);
            d.access_slot(index)?;
        }

        print!("{}", d.id);
        if status {
            let s = d.get_status();
            print!(": {:?}", s);
        }
        println!();
    }

    Ok(())
}
