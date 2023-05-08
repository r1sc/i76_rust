use std::{fs::File, io::BufWriter, path::Path};

use clap::{Parser, Subcommand};
use lib76::{
    fileparsers::{act::ACT, cbk::CBK, map::MAP, vqm::VQM},
    zfs_archive,
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to I76.ZFS
    zfs_path: String,

    #[command(subcommand)]
    command: ZFSCommand,
}

#[derive(Debug, Clone, Subcommand)]
enum ZFSCommand {
    /// Lists files in the ZFS (note these commands skip all .pix and .pak files - their contents are listed instead)
    ListFiles,
    /// Extracts a file from the ZFS
    Extract {
        filename: String,
        target_folder: String,
    },
    /// Extracts all files from the ZFS
    ExtractAll { target_folder: String },
    /// Extracts all textures from the ZFS and encodes them as PNG
    ExtractAllTextures {
        target_folder: String,
        act_filename: String,
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
    let args = Args::parse();

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
            filename,
            target_folder,
        } => {
            if let Err(e) = extract_file(&filename, &target_folder) {
                println!("Failed to extract file {}, got error {}", filename, e)
            }
        }
        ZFSCommand::ExtractAll { target_folder } => {
            let file_list = archive.get_file_list();
            for file in file_list {
                if file.ends_with(".pix") || file.ends_with(".pak") {
                    continue;
                }
                if let Err(e) = extract_file(&file, &target_folder) {
                    println!("Failed to extract file {}, got error {}", file, e)
                }
            }
        }
        ZFSCommand::ExtractAllTextures {
            target_folder,
            act_filename,
        } => {
            let act: ACT = archive.load(&act_filename)?;
            let file_list = archive.get_file_list();
            for file in file_list {
                if file.ends_with(".map") {
                    let target_path = format!("{}/{}.png", target_folder, &file);
                    let map: MAP = archive.load(&file)?;

                    let pixels = map.to_rgba_pixels(&act);
                    write_png(map.width, map.height, &pixels, &target_path)?;
                } else if file.ends_with(".vqm") {
                    let target_path = format!("{}/{}.png", target_folder, &file);
                    let vqm: VQM = archive.load(&file)?;
                    let cbk: CBK = archive.load(&vqm.cbk_filename.to_lowercase())?;

                    let pixels = vqm.to_rgba_pixels(&cbk, &act);
                    write_png(vqm.width, vqm.height, &pixels, &target_path)?;
                }
            }
        }
    }

    Ok(())
}
