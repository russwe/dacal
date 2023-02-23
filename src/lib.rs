pub mod error;
use error::{ RusbOrDacal, SpindleError };

use utf16string::WStr;
use log::debug;
use rusb::{ Device, DeviceHandle, GlobalContext };
use rustc_serialize::hex::ToHex;
use std::time::Duration;

pub fn devices() -> Result<Vec<Dacal>, rusb::Error> {
    // 0x04b4: Cypress Semiconductor Corp.
    // 0x5a9b: Dacal CD/DVD Library D-101/DC-300/DC-016RW
    // 
    // /etc/udev/rules.d/99-dacal-usb-rules.x
    //  SUBSYSTEM=="usb", ATTRS{idVendor}=="04b4", ATTRS{idProduct}=="5a9b", MODE="0660", GROUP="dacal"
    //  SUBSYSTEM=="usb_device", ATTRS{idVendor}=="04b4", ATTRS{idProduct}=="5a9b" MODE="0660", GROUP="dacal"

    Ok(rusb::devices()?
    .iter()
    .filter_map(|d| match d.device_descriptor() {
        Ok(desc) if desc.vendor_id() == 0x04b4 && desc.product_id() == 0x5a9b => Dacal::from_device(d).ok(),
        _ => None
    })
    .collect())
}

pub struct Dacal {
    pub id: u16,
    device: Device<GlobalContext>,
}

#[derive(Debug)]
pub enum DacalStatus {
    Ok,
    Busy,
    Sos,
    Unknown { status: String },
}

impl std::fmt::Display for DacalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DacalStatus::Ok => write!(f, "OK"),
            DacalStatus::Busy => write!(f, "Spindle busy, try again later."),
            DacalStatus::Sos => write!(f, "SOS, please reset Spindle!"),
            DacalStatus::Unknown { status } => write!(f, "Unknown Error Status: {}", status),
        }
    }
}

impl From<String> for DacalStatus {
    fn from(s: String) -> Self {
        match &s[..] {
            "ACK" => DacalStatus::Ok,
            "BUSY" => DacalStatus::Busy,
            _ => DacalStatus::Unknown { status: s },
        }
    }
}

impl Dacal {
    /*
        Information on DACAL is hard to find, so I'd like to honor those that came before:
        - https://sourceforge.net/projects/qcdorganizer
        - https://sourceforge.net/projects/dacal/files/Dacal/0.2-alpha/
        - https://sourceforge.net/p/libcdrom/code
        - https://sourceforge.net/p/libcdorganizer/code/HEAD/tree/trunk/libcdorganizer/src/plugin/dacal/dacal.c

        Response Structure: LL??xxxxxxxxxxxx
        LL: Total Length of Response in Bytes
        ??: ?
        xx: UTF-16LE Status ACK, BUSY, SOS, ?
    */

    const ID:u8         = 0x0a;
    const RESET:u8      = 0x0b;
    const MOVE_TO:u8    = 0x0c; // Followed by another request with disc no.
  //const ???????:u8    = 0x0d; // ? Wonder what this does...
    const RETRACT:u8    = 0x0e;
    const LED_ON:u8     = 0x0f;
    const LED_OFF:u8    = 0x10;
    const GET_STATUS:u8 = 0x11; // ! Returns string: b[2,4,6] = 'ACK' | b[2,4,6,8] = 'Busy' | SOS? | ...  (Unicode UTF-16?)
  //const SLOT_COUNT:u8 = 0x12; // [C016RW:100] ? buffer[2,4,6]
  //const INSERT:u8     = 0x13; // [C016RW:100] Followed by request with disc no.  o.O
  //const EJECT:u8      = 0x14; // [C016RW:100]

    fn from_device(device: Device<GlobalContext>) -> Result<Dacal, SpindleError> {
        let handle = device.open()?;
        let id = Dacal::get_id(&handle)?;
        Ok(Dacal { id, device })
    }

    pub fn from_id(id: u16) -> Result<Dacal, SpindleError> {
        devices()?.into_iter().find(|d| d.id == id).ok_or(SpindleError::NoSpindle { id })
    }

    pub fn retract_arm(&self) -> Result<(), SpindleError> {
        self.execute_rusb(|h| {
            Dacal::issue_command(h, Dacal::RETRACT)
        })
    }

    pub fn access_slot(&self, slot_number: u8) -> Result<(), SpindleError> {
        if slot_number < 1 || slot_number > 150 {
            return Err(SpindleError::NoSlot { id: self.id, number: slot_number });
        }

        self.execute_rusb(|h| {
            Dacal::issue_command(h, Dacal::RETRACT)?;

            Dacal::issue_command(h, Dacal::MOVE_TO)?;
            Dacal::issue_command(h, slot_number)
        })
    }

    pub fn get_status(&self) -> Result<DacalStatus, SpindleError> {
        let r = self.execute_rusb(|h| {
            Dacal::issue_command(h, Dacal::GET_STATUS)
        });

        match r.err() {
            None => Ok(DacalStatus::Ok),
            Some(SpindleError::ErrorStatus { status }) => Ok(status),
            Some(e) => Err(e),
        }
    }

    pub fn reset(&self) -> Result<(), SpindleError> {
        self.execute_rusb(|h| {            
            Dacal::issue_command(h, Dacal::RESET)
        })
    }

    pub fn set_led(&self, on: bool) -> Result<(), SpindleError> {
        let command = if on { Dacal::LED_ON } else { Dacal::LED_OFF };

        self.execute_rusb(|h| {
            Dacal::issue_command(h, command)
        })
    }

    fn execute_rusb<C: FnOnce(&DeviceHandle<GlobalContext>) -> Result<(), RusbOrDacal>>(&self, cmds: C) -> Result<(), SpindleError> {
        self.execute_rusb_and_translate(cmds, |_,_| None)
    }

    fn execute_rusb_and_translate<C : FnOnce(&DeviceHandle<GlobalContext>) -> Result<(), RusbOrDacal>, I: FnOnce(&Dacal, rusb::Error) -> Option<SpindleError>>(&self, cmds: C, into: I) -> Result<(), SpindleError> {
        let handle = self.device.open()?;
        cmds(&handle).map_err(|e| match e {
            RusbOrDacal::Rusb { error: rusb::Error::Busy } => SpindleError::Busy { id: self.id },
            RusbOrDacal::Dacal { status: DacalStatus::Busy } => SpindleError::Busy {id: self.id },

            RusbOrDacal::Rusb { error } => into(self, error).unwrap_or_else(|| error.into()),
            RusbOrDacal::Dacal { status } => SpindleError::ErrorStatus { status },
        })
    }

    pub fn debug(&self, command: u8) -> Result<(), SpindleError> {
        self.execute_rusb(|h| {
            Dacal::issue_command(&h, command)
        })
    }

    fn get_id(handle:&DeviceHandle<GlobalContext>) -> rusb::Result<u16> {
        let mut buff = [0x00;8];
        let len = handle.read_control(
            rusb::request_type(rusb::Direction::In, rusb::RequestType::Standard, rusb::Recipient::Device),
            0x06,
            0x03 << 8 | u16::from(Dacal::ID),
            1033,
            &mut buff,
            Duration::from_secs(1)
        )?;

        debug!("030A ({}): {}", len, &buff[0..len].to_hex());

        if len < 7 {
            return Err(rusb::Error::Other);
        }

        Ok(u16::from(buff[4]) << 8 | u16::from(buff[6]))
    }
    
    fn issue_command(handle:&DeviceHandle<GlobalContext>, index:u8) -> Result<(), RusbOrDacal> {
        let mut buff = [0x00;255];
        let len = handle.read_control(
            rusb::request_type(rusb::Direction::In, rusb::RequestType::Standard, rusb::Recipient::Device),
            0x06,
            0x03 << 8 | u16::from(index),
            1033,
            &mut buff,
            Duration::from_secs(1)
        )?;
    
        let rstatus = WStr::from_utf16le(&buff[2..len]).map_or_else(
            |_| "Parse Error".to_string(),
            |s| s.to_string()
        );
        debug!("03{:02X} ({}): {} '{}'", index, len, &buff[0..len].to_hex(), rstatus);

        match rstatus.into() {
            DacalStatus::Ok => Ok(()),
            status => Err(RusbOrDacal::Dacal { status })
        }
    }
}