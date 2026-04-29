use clap::{Parser, Subcommand};
use ndsdplayback::players::{create_player, enumerate_supported_devices};
use std::time::Duration;
use tokio::time::sleep;


///ndsd-play is a tool to play dsd file through sinks with support of native dsd
/// To play a file you need to get your device id firstly, use list-devices command and remember id of your device
///
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    ///Enumerate supported devices in system
    ListDevices,
    ///Play a file through(device_id device_id can be obtained from list-devices command)
    /// Example: ndsd-play open <device_id> <path_to_file>
    /// Real world example: ndsd-play open 1 "/mnt/hdd/Music/Alphaville - Forever Young (Remastered) (1984_2019) [LP] DSD128/Alphaville - Forever Young (Remastered) (1984_2019) [LP] DSD128.dff"
    Open { device: u32, file: String },
    ///Displays the current version of ndsd-play
    Version
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::ListDevices => {
            let devices = enumerate_supported_devices();
            println!("");
            println!("");
            println!("");

            if devices.is_empty() {
                println!("NO SUITABLE DEVICES FOUND");
            } else {
                println!("FOUND {} SUITABLE DEVICES", devices.len());
                for i in 0..devices.len() {
                    println!(
                        "Id: {} Short name: {} Full name: {}",
                        i,
                        devices[i].0.to_string_lossy(),
                        devices[i].1.to_string_lossy()
                    );
                }
            }
        }
        Command::Open { device, file } => {
            open_file_and_play(file, device).await;
        },
        Command::Version => {
            println!("ndsd-play version is: {}, dst decoding is supported", env!("CARGO_PKG_VERSION"));
        }
    }
}

async fn open_file_and_play(path: String, device: u32) {
    let devices = enumerate_supported_devices();
    if device >= devices.len() as u32 {
        eprintln!("Device not found or busy");
        return;
    }

    let player = create_player(devices[device as usize].0.clone());
    if player.is_none() {
        eprintln!("Failed to create player with current device");
        return;
    }
    let mut player = player.unwrap();
    player.load_new_track(path.as_str()).await;
    player.start().await;
    sleep(Duration::from_millis(2500)).await;
    if let Some(meta) = player.get_current_file_meta().await{
        meta.pretty_print();
    }
    println!("Control playback by entering commands from the list below");
    println!(
        "Controls: (Pause/Play - p), (Seek - enter number in float like(0.1-0.9)), (Get current progress - e), (Stop - s), (Print current track data - m)"
    );
    let mut paused = false;
    while player.is_playing().await {
        let mut input = String::new();
        input = input.to_lowercase();
        std::io::stdin().read_line(&mut input).expect("Failed");
        let mut chars = input.chars();
        match chars.nth(0).unwrap() {
            'p' => {
                if !paused {
                    player.pause().await;
                } else {
                    player.play().await;
                }
                paused = !paused;
            }
            '0' => {
                let seek: Result<f64, _> = input.trim().parse();
                match seek {
                    Ok(seek) => {
                        if seek >= 0f64 && seek <= 1f64 {
                            let res = player.seek(seek).await;
                            if res.is_err() {
                                eprintln!("Seek failed {}", res.unwrap_err());
                            }
                        } else {
                            eprintln!("Invalid number entered");
                        }
                    }
                    Err(err) => {
                        eprintln!("Invalid number entered {} {}", err, input);
                    }
                }
            }
            'e' => {
                println!("Progress: {}", player.get_pos().await);
            }
            's' => {
                player.stop().await;
                break;
            }
            'm' => {
                if let Some(meta) = player.get_current_file_meta().await{
                    meta.pretty_print();
                } else {
                    println!("Failed to get current track data");
                }
                println!("Format is {:?}", player.get_format_info().await);
            }
            _ => {
                eprintln!("invalid command");
            }
        }
    }
    drop(player);
}
