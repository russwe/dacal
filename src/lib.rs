extern crate rustc_serialize;

use rusb::{ Device, DeviceHandle, GlobalContext };
use std::time::Duration;
use rustc_serialize::hex::ToHex;

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
    Ok(),
    Sos(),
}

impl Dacal {
    const ID:u16 = 0x0a;

    // Command-Codes
    const MOVE_TO:u8    = 0x0c; // Followed by another request with disc no.
    const RETRACT:u8    = 0x0e;
  //const GET_STATUS:u8 = 0x00; // ?

    fn from_device(device: Device<GlobalContext>) -> Result<Dacal, rusb::Error> {
        if let Ok(handle) = device.open() {
            return Dacal::get_id(&handle).map(|id| Dacal { id, device });
        } else {
            return Err(rusb::Error::Other);
        }
    }

    pub fn from_id(id: u16) -> Result<Dacal, rusb::Error> {
        devices()?.into_iter().find(|d| d.id == id).ok_or(rusb::Error::BadDescriptor)
    }

    pub fn retract_arm(&self) -> Result<(), rusb::Error> {
        let handle = self.device.open()?;
        Dacal::issue_command(&handle, Dacal::RETRACT)?;
        return Ok(());
    }

    pub fn access_slot(&self, slot_number: u8) -> Result<(), rusb::Error> {
        let handle = self.device.open()?;

        Dacal::issue_command(&handle, Dacal::RETRACT)?;
        
        Dacal::issue_command(&handle, Dacal::MOVE_TO)?;
        Dacal::issue_command(&handle, slot_number)?;

        return Ok(());
    }

    pub fn get_status(&self) -> DacalStatus {
        return DacalStatus::Ok();
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

        //TODO: Figure out how to enable this in debug / verbose mode (cleanly)
        println!("030A ({}): {}", len, &buff[0..len].to_hex());

        if len < 7 {
            return Err(rusb::Error::Other);
        }

        return Ok(u16::from(buff[4]) << 8 | u16::from(buff[6]));
    }
    
    fn issue_command(handle:&DeviceHandle<GlobalContext>, index:u8) -> Result<(), rusb::Error> {
        let mut buff = [0x00;8];
        let len = handle.read_control(
            rusb::request_type(rusb::Direction::In, rusb::RequestType::Standard, rusb::Recipient::Device),
            0x06,
            0x03 << 8 | u16::from(index),
            1033,
            &mut buff,
            Duration::from_secs(1)
        )?;
    
        //TODO: Figure out how to enable this in debug / verbose mode (cleanly)
        println!("03{:02X} ({}): {}", index, len, &buff[0..len].to_hex());

        return Ok(());
    }
}
