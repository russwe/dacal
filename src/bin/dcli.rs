use std::{error::Error, time::Duration, thread, io, io::Write };

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
    Blink {
        spindle_id: u16,
    },
    Close {
        spindle_id: u16,
    },
    Debug {
        spindle_id: u16,
        command: u8,
    },
    List {
        #[arg(short, long)]
        identify: bool,

        #[arg(short, long)]
        status: bool,
    },
    Open {
        spindle_id: u16,
        disc_number: u8,
    },
    Reset {
        spindle_id: u16,
    },
    Status {
        spindle_id: u16,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Blink { spindle_id } => cmd_blink(*spindle_id),
        Commands::Close { spindle_id } => cmd_close(*spindle_id),
        Commands::Debug { spindle_id, command } => cmd_debug(*spindle_id, *command),
        Commands::List { identify, status } => cmd_list(*identify, *status),
        Commands::Open { spindle_id, disc_number } => cmd_open(*spindle_id, *disc_number),
        Commands::Reset { spindle_id } => cmd_reset(*spindle_id),
        Commands::Status { spindle_id } => cmd_status(*spindle_id),
    }
}

fn cmd_blink(spindle_id: u16) -> Result<(), Box<dyn Error>> {
    let d = Dacal::from_id(spindle_id)?;

    let mut count = 3;
    while count > 0 {
        d.set_led(true)?;
        print!("+"); io::stdout().flush()?;
        thread::sleep(Duration::from_secs(1));

        d.set_led(false)?;
        print!("-"); io::stdout().flush()?;
        thread::sleep(Duration::from_secs(1));

        count -= 1;
    }

    Ok(())
}

fn cmd_close(spindle_id: u16) -> Result<(), Box<dyn Error>> {
    let d = Dacal::from_id(spindle_id)?;
    d.retract_arm()?;

    Ok(())
}

fn cmd_debug(spindle_id: u16, command: u8) -> Result<(), Box<dyn Error>> {
    let d = Dacal::from_id(spindle_id)?;
    match d.debug(command) {
        Ok(()) => println!("OK"),
        Err(e) => println!("{}", e),
    }

    Ok(())
}

fn cmd_open(spindle_id: u16, disc_number: u8) -> Result<(), Box<dyn Error>> {
    let d = Dacal::from_id(spindle_id)?;
    d.access_slot(disc_number)?;

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
            let s = d.get_status()?;
            print!(": {}", s);
        }
        println!();
    }

    Ok(())
}

fn cmd_reset(spindle_id: u16) -> Result<(), Box<dyn Error>> {
    let d = Dacal::from_id(spindle_id)?;
    d.reset()?;

    Ok(())
}

fn cmd_status(spindle_id: u16) -> Result<(), Box<dyn Error>> {
    let d = Dacal::from_id(spindle_id)?;
    let status = d.get_status()?;

    println!("{}: {}", spindle_id, status);

    Ok(())
}
