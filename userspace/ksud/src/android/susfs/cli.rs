use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::android::susfs::api::{self};

#[derive(Debug, Args)]
pub struct SusfsArgs {
    #[command(subcommand)]
    pub command: SuSFSSubCommands,
}

#[derive(Debug, Subcommand)]
pub enum SuSFSSubCommands {
    ///  Added path and all its sub-paths will be hidden for umounted app process from several syscalls
    /// Please be reminded that if the target path has upper mounts then make sure the proper layer is added, otherwise it may not be effective for the target process.
    #[command(name = "add_sus_path")]
    AddSusPath {
        #[arg(help = "Path of file or directory")]
        path: String,
    },
    ///  The only difference to add_sus_path is that the added sus_path via this cli will be flagged as SUS_PATH again for the app process when it is being spawned by zygote and marked umounted
    /// Also it does not check if the path is existed or not, instead it checks for empty string only, so be careful what to add.
    AddSusPathLoop {
        #[arg(help = "Path not inside sdcard")]
        path: String,
    },
    /// Add path to store original stat info in kernel memory (before bind mount/overlay)
    #[command(name = "add_sus_kstat")]
    AddSusKstat {
        /// Add the desired path BEFORE it gets bind mounted or overlayed, this is used for storing original stat info in kernel memory
        /// This command must be completed with <update_sus_kstat> later after the added path is bind mounted or overlayed
        path: String,
    },
    /// Update the target ino for a path added via add_sus_kstat
    #[command(name = "update_sus_kstat")]
    UpdateSusKstat {
        /// Add the desired path you have added before via <add_sus_kstat> to complete the kstat spoofing procedure\n");
        /// This updates the target ino, but size and blocks are remained the same as current stat
        path: String,
    },
    /// Update target ino only, other stat members remain same as original
    #[command(name = "update_sus_kstat_full_clone")]
    UpdateSusKstatFullClone {
        /// Add the desired path you have added before via <add_sus_kstat> to complete the kstat spoofing procedure
        /// This updates the target ino only, other stat members are remained the same as the original stat
        path: String,
    },
    /// Spoof uname for all processes, set string to 'default' to imply the function to use original string
    #[command(name = "set_uname")]
    SetUname { release: String, version: String },
    /// Enable/Disable susfs log in kernel
    #[command(name = "enable_log")]
    EnableLog {
        /// 0: disable susfs log in kernel
        /// 1: enable susfs log in kernel
        enabled: u8,
    },
    /// Spoof /proc/cmdline or /proc/bootconfig
    #[command(name = "set_cmdline_or_bootconfig")]
    SetCmdlineOrBootconfig { path: String },
    /// Redirect target path to be opened with user defined path
    #[command(name = "add_open_redirect")]
    AddOpenRedirect {
        target_path: String,
        redirected_path: String,
        ///0: Effective for non-app processes (uid < 10000
        ///1: Effective for non-su processes of which uid is 0 (All root process but not with su domain)
        ///2: Effective for non-su processes (Use it carefully!)
        ///3: Effective for processes that are marked umounted with uid >= 10000 (Use it carefully!)
        ///4: Effective for processes that are marked umounted (include most of the init spawned process, use it carefully!)
        uid_scheme: u64,
    },
    /// Added real file path which gets mmapped will be hidden from /proc/self/[maps|smaps|smaps_rollup|map_files|mem|pagemap]
    #[command(name = "add_sus_map")]
    AddSusMap { path: String },
    /// Enable/Disable spoofing sus 'su' context in avc log
    #[command(name = "enable_avc_log_spoofing")]
    EnableAvcLogSpoofing {
        /// 0: disable spoofing the sus tcontext 'su' shown in avc log in kernel
        /// 1: enable spoofing the sus tcontext 'su' with 'u:r:priv_app:s0:c512,c768' shown in avc log in kernel
        enabled: u8,
    },
    /// Show version, enabled_features, or variant
    Show {
        /// version: show the current susfs version implemented in kernel
        /// enabled_features: show the current implemented susfs features in kernel
        /// variant: show the current variant: GKI or NON-GKI
        #[command(subcommand)]
        info_type: ShowType,
    },
    /// (Advanced) Add sus kstat statically with manual or default values
    #[command(name = "add_sus_kstat_statically")]
    AddSusKstatStatically(Box<AddSusKstatStaticallyArgs>),
}

#[derive(Subcommand, Debug)]
pub enum ShowType {
    Version,
    EnabledFeatures,
    Variant,
}

#[derive(Debug, Args)]
pub struct AddSusKstatStaticallyArgs {
    path: String,
    #[arg(default_value = "default")]
    ino: String,
    #[arg(default_value = "default")]
    dev: String,
    #[arg(default_value = "default")]
    nlink: String,
    #[arg(default_value = "default")]
    size: String,
    #[arg(default_value = "default")]
    atime: String,
    #[arg(default_value = "default")]
    atime_nsec: String,
    #[arg(default_value = "default")]
    mtime: String,
    #[arg(default_value = "default")]
    mtime_nsec: String,
    #[arg(default_value = "default")]
    ctime: String,
    #[arg(default_value = "default")]
    ctime_nsec: String,
    #[arg(default_value = "default")]
    blocks: String,
    #[arg(default_value = "default")]
    blksize: String,
}

#[derive(Debug, Parser)]
struct SusfsParser {
    #[command(flatten)]
    arg: SusfsArgs,
}

pub fn run_from_args(args: &[String]) -> Result<()> {
    let parser = SusfsParser::try_parse_from(args)?;
    run_main(parser.arg.command)
}

pub fn run_main(command: SuSFSSubCommands) -> Result<()> {
    //let command = SuSFSSubCommands::try_parse_from(args)?;
    match command {
        SuSFSSubCommands::AddSusPath { path } => {
            api::add_sus_path(&api::SusPathType::Normal, &path)?;
        }
        SuSFSSubCommands::AddSusPathLoop { path } => {
            api::add_sus_path(&api::SusPathType::Loop, &path)?;
        }
        SuSFSSubCommands::AddSusKstat { path } => {
            api::add_sus_kstat(path)?;
        }
        SuSFSSubCommands::UpdateSusKstat { path } => {
            api::update_sus_kstat(path)?;
        }
        SuSFSSubCommands::UpdateSusKstatFullClone { path } => {
            api::update_sus_kstat_full_clone(path)?;
        }
        SuSFSSubCommands::SetUname { release, version } => {
            api::set_uname(&release, &version)?;
        }
        SuSFSSubCommands::EnableLog { enabled } => {
            api::enable_log(enabled)?;
        }
        SuSFSSubCommands::SetCmdlineOrBootconfig { path } => {
            api::set_cmdline_or_bootconfig(path)?;
        }
        SuSFSSubCommands::AddOpenRedirect {
            target_path,
            redirected_path,
            uid_scheme,
        } => {
            api::add_open_redirect(target_path, redirected_path, uid_scheme)?;
        }
        SuSFSSubCommands::AddSusMap { path } => {
            api::add_sus_map(path)?;
        }
        SuSFSSubCommands::EnableAvcLogSpoofing { enabled } => {
            api::enable_avc_log_spoofing(enabled)?;
        }
        SuSFSSubCommands::Show { info_type } => match info_type {
            ShowType::Version => {
                api::show_version()?;
            }
            ShowType::EnabledFeatures => {
                api::show_features()?;
            }
            ShowType::Variant => {
                api::show_variant()?;
            }
        },
        SuSFSSubCommands::AddSusKstatStatically(args) => {
            api::add_sus_kstat_statically(
                &args.path,
                &args.ino,
                &args.dev,
                &args.nlink,
                &args.size,
                &args.atime,
                &args.atime_nsec,
                &args.mtime,
                &args.mtime_nsec,
                &args.ctime,
                &args.ctime_nsec,
                &args.blocks,
                &args.blksize,
            )?;
        }
    }

    Ok(())
}
