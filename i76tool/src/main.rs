use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use clap::{Parser, Subcommand};
use lib76::{
    fileparsers::{
        act::ACT,
        binary_reader::{BinaryReader, Readable},
        cbk::CBK,
        map::MAP,
        msn::{MSN, FSMOpcode},
        vqm::VQM,
    },
    zfs_archive,
};
use wax::{Glob, Pattern};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: ZFSCommand,
}

#[derive(Debug, Subcommand)]
enum ZFSCommand {
    /// Lists files in the ZFS (note these commands skip all .pix and .pak files - their contents are listed instead)
    ListFiles {
        /// Path to I76.ZFS
        zfs_path: String,
    },
    /// Extracts files from the ZFS by pattern
    Extract {
        /// Path to I76.ZFS
        zfs_path: String,
        pattern: String,
        target_folder: String,
        act_filename: Option<String>,
    },
    /// Decompile mission script from mission file
    Decompile { msn_path: String },
}

pub fn convert(data: &[u32]) -> Vec<u8> {
    let mut res = Vec::with_capacity(data.len() * 4);
    for &d in data {
        res.push((d >> 16) as u8);
        res.push((d >> 8) as u8);
        res.push(d as u8);
        res.push(255);
    }
    res
}

fn main() -> Result<(), std::io::Error> {
    let args = Cli::parse();

    let write_png =
        |width: u32, height: u32, data: &[u32], target_path: &str| -> Result<(), std::io::Error> {
            let path = Path::new(target_path);
            let file = File::create(path).unwrap();
            let w = BufWriter::new(file);
            let mut encoder = png::Encoder::new(w, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;

            writer.write_image_data(&convert(data))?;

            Ok(())
        };

    let extract_file = |archive: &zfs_archive::ZFSArchive,
                        which: &str,
                        target_folder: &str|
     -> Result<(), std::io::Error> {
        let target_path = format!("{}/{}", target_folder, which);
        let data = archive.get_archive_data(which)?;
        std::fs::write(target_path, data)?;

        Ok(())
    };

    match args.command {
        ZFSCommand::ListFiles { zfs_path } => {
            let archive = zfs_archive::ZFSArchive::new(Path::new(&zfs_path))?;

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
            zfs_path,
            pattern,
            target_folder,
            act_filename,
        } => {
            let archive = zfs_archive::ZFSArchive::new(Path::new(&zfs_path))?;
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
                            let pixels = map.to_rgba_pixels(act.as_ref().unwrap(), true);
                            write_png(map.width, map.height, &pixels, &target_path)?;
                        }
                        Err(e) => println!("Failed to extract file {}, got error {}", file, e),
                    }
                } else if file.ends_with(".vqm") && act.is_some() {
                    let target_path = format!("{}/{}.png", target_folder, &file);

                    match archive.load::<VQM>(&file) {
                        Ok(vqm) => {
                            let cbk: CBK = archive.load(&vqm.cbk_filename.to_lowercase())?;
                            let pixels = vqm.to_rgba_pixels(&cbk, act.as_ref().unwrap(), true);
                            write_png(vqm.width, vqm.height, &pixels, &target_path)?;
                        }
                        Err(e) => println!("Failed to extract file {}, got error {}", file, e),
                    }
                } else if let Err(e) = extract_file(&archive, &file, &target_folder) {
                    println!("Failed to extract file {}, got error {}", file, e)
                }
            }
        }
        ZFSCommand::Decompile { msn_path } => {
            let mut br = BinaryReader {
                reader: BufReader::new(Box::new(File::open(msn_path)?)),
            };
            let msn = MSN::consume(&mut br)?;

            println!("[Actions]");
            for (i, action) in msn.fsm.action_table.iter().enumerate() {
                println!("{} {}", i, action);
            }
            println!();

            println!("[SoundClips]");
            for (i, clip) in msn.fsm.sound_clip_table.iter().enumerate() {
                println!("{} {}", i, clip);
            }
            println!();

            println!("[Entities]");
            for (i, entity) in msn.fsm.entity_table.iter().enumerate() {
                println!("{} {} {} {}", i, entity.id, entity.label, entity.value);
            }
            println!();

            println!("[Paths]");
            for (i, path) in msn.fsm.paths.iter().enumerate() {
                let points: Vec<_> = path
                    .points
                    .iter()
                    .map(|p| format!("({}, {}, {})", p.x, p.y, p.z))
                    .collect();

                println!("{} {} [{}]", i, path.name, points.join(", "));
            }
            println!();

            println!("[Constants]");
            for c in msn.fsm.constants {
                println!("{}", c);
            }
            println!();

            println!("[Machines]");
            for (i, machine) in msn.fsm.stack_machine_definitions.iter().enumerate() {
                println!("{} {} {:?}", i, machine.start_address, &machine.initial_arguments);
            }
            println!();

            println!("[Bytecode]");
            for (i, instruction) in msn.fsm.raw_instructions.iter().enumerate() {
                let instruction_str = match instruction.opcode {
                    FSMOpcode::Push => format!("push {}", instruction.value),
                    FSMOpcode::ArgPushS => format!("push_s {}", instruction.value),
                    FSMOpcode::ArgPushB => format!("push_b {}", instruction.value),
                    FSMOpcode::Adjust => format!("adjust {}", instruction.value),
                    FSMOpcode::Drop => format!("drop {}", instruction.value),
                    FSMOpcode::Jmp => format!("jmp {}", instruction.value),
                    FSMOpcode::Jz => format!("jz {}", instruction.value),
                    FSMOpcode::JmpI => format!("jmp_i {}", instruction.value),
                    FSMOpcode::Rst => format!("rst {}", instruction.value),
                    FSMOpcode::Action => format!("action {}", &msn.fsm.action_table[instruction.value as usize]),
                    FSMOpcode::Neg => format!("neg {}", instruction.value)
                };
                println!("{} {}", i, instruction_str);
            }
            println!();
        }
    }

    Ok(())
}
