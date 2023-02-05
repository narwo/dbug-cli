use clap::{ArgAction, Parser, Subcommand};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, ContentArrangement, Table,
};
use probe_rs::{
    architecture::arm::{ApAddress, DpAddress},
    Probe,
};

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
                ap: 1,
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
                ap: 1,
                dp: DpAddress::Default,
            };
            const APPROTECTSTATUS: u8 = 0x0C;

            let ap_protect_status = match iface.read_raw_ap_register(CTRL_AP, APPROTECTSTATUS) {
                Ok(0) => "Locked",
                Ok(1) => "Unlocked",
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
                ap: 1,
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
    }
}
