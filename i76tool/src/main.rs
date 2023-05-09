use std::{fs::File, io::BufWriter, path::Path};

use clap::{Args, Parser, Subcommand};
use lib76::{
    fileparsers::{act::ACT, cbk::CBK, map::MAP, vqm::VQM},
    zfs_archive,
};
use wax::{Glob, Pattern};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to I76.ZFS
    zfs_path: String,

    #[command(subcommand)]
    command: ZFSCommand,
}

#[derive(Debug, Subcommand)]
enum ZFSCommand {
    /// Lists files in the ZFS (note these commands skip all .pix and .pak files - their contents are listed instead)
    ListFiles,
    /// Extracts files from the ZFS by pattern
    Extract {
        pattern: String,
        target_folder: String,
        act_filename: Option<String>,
    },
}

pub fn convert(data: &[u32]) -> Vec<u8> {
    let mut res = Vec::with_capacity(data.len() * 4);
    for i in 0..data.len() {
        res.push((data[i] >> 16) as u8);
        res.push((data[i] >> 8) as u8);
        res.push(data[i] as u8);
        res.push(255);
    }
    res
}

fn main() -> Result<(), std::io::Error> {
    let args = Cli::parse();

    let archive = zfs_archive::ZFSArchive::new(args.zfs_path)?;

    let write_png =
        |width: u32, height: u32, data: &[u32], target_path: &str| -> Result<(), std::io::Error> {
            let path = Path::new(target_path);
            let file = File::create(path).unwrap();
            let ref mut w = BufWriter::new(file);
            let mut encoder = png::Encoder::new(w, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;

            writer.write_image_data(&convert(data))?;

            Ok(())
        };

    let extract_file = |which: &str, target_folder: &str| -> Result<(), std::io::Error> {
        let target_path = format!("{}/{}", target_folder, which);
        let data = archive.get_archive_data(which)?;
        let _ = std::fs::write(&target_path, data)?;

        Ok(())
    };

    match args.command {
        ZFSCommand::ListFiles => {
            let mut file_list = archive.get_file_list();
            file_list.sort();
            for file in file_list {
                if file.ends_with(".pix") || file.ends_with(".pak") {
                    continue;
                }
                println!("{}", file);
            }
        }
        ZFSCommand::Extract {
            pattern,
            target_folder,
            act_filename,
        } => {
            let glob = Glob::new(&pattern).unwrap();

            let act: Option<ACT> = act_filename
                .map(|act_filename| archive.load(&act_filename).expect("Failed to load ACT"));

            let file_list = archive.get_file_list();
            for file in file_list {
                if file.ends_with(".pix") || file.ends_with(".pak") {
                    continue;
                }
                if !glob.is_match(&file[..]) {
                    continue;
                }

                if file.ends_with(".map") && act.is_some() {
                    let target_path = format!("{}/{}.png", target_folder, &file);
                    
                    match archive.load::<MAP>(&file) {
                        Ok(map) => {
                            let pixels = map.to_rgba_pixels(&act.as_ref().unwrap());
                            write_png(map.width, map.height, &pixels, &target_path)?;
                        }
                        Err(e) => println!("Failed to extract file {}, got error {}", file, e),
                    }
                } else if file.ends_with(".vqm") && act.is_some() {
                    let target_path = format!("{}/{}.png", target_folder, &file);

                    match archive.load::<VQM>(&file) {
                        Ok(vqm) => {
                            let cbk: CBK = archive.load(&vqm.cbk_filename.to_lowercase())?;
                            let pixels = vqm.to_rgba_pixels(&cbk, &act.as_ref().unwrap());
                            write_png(vqm.width, vqm.height, &pixels, &target_path)?;
                        }
                        Err(e) => println!("Failed to extract file {}, got error {}", file, e),
                    }
                } else {
                    if let Err(e) = extract_file(&file, &target_folder) {
                        println!("Failed to extract file {}, got error {}", file, e)
                    }
                }
            }
        }
    }

    Ok(())
}
