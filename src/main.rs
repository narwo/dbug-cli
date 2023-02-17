use clap::{ArgAction, Parser, Subcommand};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, ContentArrangement, Table,
};
use packed_struct::prelude::*;
use probe_rs::{
    architecture::arm::{ApAddress, DpAddress},
    Probe,
};
use svd_parser as svd;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "dbug")]
#[command(about = "Command-line tool for D'Bug", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, action=ArgAction::SetTrue, help="More verbose output")]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List probes
    List {},
    /// Reset target
    Reset {},
    /// Target status
    Status {},
    /// Unlock target, erases all
    Unlock {},
    /// Lock target
    Lock {},
    /// Show Registers (Temporary command)
    Registers {},
}

#[derive(Debug, PackedStruct, Copy, Clone, PartialEq)]
#[packed_struct(size_bytes = "4", bit_numbering = "lsb0")]
pub struct IDR {
    #[packed_field(bits = "0:7", endian = "msb")]
    apid: Integer<u8, packed_bits::Bits<8>>,
    #[packed_field(bits = "13:16", endian = "msb")]
    class: Integer<u8, packed_bits::Bits<4>>,
    #[packed_field(bits = "17:23", endian = "msb")]
    jep106id: Integer<u8, packed_bits::Bits<7>>,
    #[packed_field(bits = "24:27", endian = "msb")]
    jep106cont: Integer<u8, packed_bits::Bits<4>>,
    #[packed_field(bits = "28:31", endian = "msb")]
    revision: Integer<u8, packed_bits::Bits<4>>,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::List {} => {
            let probes = Probe::list_all();
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Name").fg(Color::Green),
                    Cell::new("Type").fg(Color::Green),
                    Cell::new("Serial").fg(Color::Green),
                ]);

            for probe in &probes {
                table.add_row(vec![
                    Cell::new(&probe.identifier),
                    Cell::new(format!("{:?}", probe.probe_type)),
                    Cell::new(&probe.serial_number.clone().unwrap()),
                ]);
            }
            println!("{table}");
        }
        Commands::Reset {} => {
            let probes = Probe::list_all();
            let mut probe = probes[0].open().unwrap();
            probe.attach_to_unspecified().unwrap();
            let mut iface = probe
                .try_into_arm_interface()
                .unwrap()
                .initialize_unspecified()
                .unwrap();
            const CTRL_AP: ApAddress = ApAddress {
                ap: 4,
                dp: DpAddress::Default,
            };
            const RESET: u8 = 0x0;
            iface.write_raw_ap_register(CTRL_AP, RESET, 1).unwrap();
            iface.write_raw_ap_register(CTRL_AP, RESET, 0).unwrap();
        }
        Commands::Status {} => {
            let probes = Probe::list_all();
            let mut probe = probes[0].open().unwrap();
            probe.attach_to_unspecified().unwrap();
            let mut iface = probe
                .try_into_arm_interface()
                .unwrap()
                .initialize_unspecified()
                .unwrap();
            const CTRL_AP: ApAddress = ApAddress {
                ap: 4,
                dp: DpAddress::Default,
            };
            const APPROTECTSTATUS: u8 = 0x0C;
            const IDR: u8 = 0xFC;

            let idr = iface
                .read_raw_ap_register(CTRL_AP, IDR)
                .unwrap()
                .to_be_bytes();
            let idr = IDR::unpack(&idr).unwrap();
            println!("{:?}", idr);

            let jep106_manufacturer =
                jep106::JEP106Code::new(idr.jep106cont.into(), idr.jep106id.into()).get();
            println!("JEP106: {}", jep106_manufacturer.unwrap());

            let ap_protect_status = match iface.read_raw_ap_register(CTRL_AP, APPROTECTSTATUS) {
                Ok(0) => "Locked : Secure_Locked",
                Ok(1) => "Unlocked : Secure_Locked",
                Ok(2) => "Locked : Secure_Unlocked",
                Ok(3) => "Unlocked : Secure_Unlocked",
                _ => "Unknown",
            };
            println!("AP Protect State: {}", ap_protect_status);

        }
        Commands::Unlock {} => {
            let probes = Probe::list_all();
            let mut probe = probes[0].open().unwrap();
            probe.attach_to_unspecified().unwrap();
            println!("Waiting to erase...");
            println!("Erasing");
            let mut iface = probe
                .try_into_arm_interface()
                .unwrap()
                .initialize_unspecified()
                .unwrap();
            const CTRL_AP: ApAddress = ApAddress {
                ap: 4,
                dp: DpAddress::Default,
            };
            const ERASEALL: u8 = 0x04;
            const ERASEALLSTATUS: u8 = 0x08;
            iface.write_raw_ap_register(CTRL_AP, ERASEALL, 1).unwrap();
            while iface.read_raw_ap_register(CTRL_AP, ERASEALLSTATUS).unwrap() != 1 {}
            println!("Erase complete");
        }
        Commands::Lock {} => {
            unimplemented!("Lock");
        }
        Commands::Registers {} => {
            let xml = &mut String::new();
            let _ = File::open("nrf52840.svd").unwrap().read_to_string(xml);
            let device = svd::parse(xml).unwrap();
            dbg!(&device);
            let uicr = device.get_peripheral("UICR").unwrap();
            let uicr_registers = uicr.registers();
        
            let mut uicr_registers_table = Table::new();
            uicr_registers_table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Name").fg(Color::Green),
                    Cell::new("Display Name").fg(Color::Green),
                    Cell::new("Description").fg(Color::Green),
                    Cell::new("Address Offset").fg(Color::Green),
                ]);
        
            for uicr_register in uicr_registers {
                uicr_registers_table.add_row(vec![
                    Cell::new(&uicr_register.name),
                    Cell::new(format!("{:?}", &uicr_register.properties)),
                    Cell::new(format!("{:?}", &uicr_register.description)),
                    Cell::new(&uicr_register.address_offset),
                ]);
            }
            // println!("{uicr_registers_table}");
        }
    }
}
