use std::path::PathBuf;

use datarace_plugin_api::wrappers::PluginHandle;
use proton_finder::GameDrive;

const BRIDGE_EXE_NAME:&'static str = "shm-bridge-rf2.exe";

/// Checks if requirements are met
/// If not install the software
pub(crate) fn init_setup(handle: &PluginHandle) -> Result<(), String> {
    let prefix = match proton_finder::get_game_drive(365960) {
        Ok(res) => res,
        Err(res) => {
            handle.log_error("$STEAM_DIR was set, but no steam install found there!");
            res
        }
    }.ok_or("rF2 Prefix not found! Make sure to install and launch the game at least once!".to_string())?;

    let bridge_path = shm_bridge_path(&prefix).ok_or("Unable to find User/AppData/Local/DataRace within the rf2 prefix!".to_string())?;

    if !bridge_path.exists() {
        // Installing bridge
        
        // TODO
        // return Err("bridge missing".to_string());
    }

    
    // TODO check if bridge is already running
    // Check if the game is running
    

    // TODO install rF2SharedMemoryMap Plugin if not present
    // And make sure it is enabled, for which we need to parse
    // UserData/player/CustomPluginVariables.json

    

    Ok(())
}

fn shm_bridge_path(prefix: &GameDrive) -> Option<PathBuf> {
    let mut path = prefix.config_local_dir()?;
    
    path.push("DataRace");
    if !path.exists() {
        std::fs::create_dir(path.as_path()).ok()?;
    }
    path.push(BRIDGE_EXE_NAME);

    Some(path)
}


