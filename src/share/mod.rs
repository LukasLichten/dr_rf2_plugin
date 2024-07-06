use std::{ffi::{c_void, CString}, marker::PhantomData, mem::size_of, os::fd::{AsRawFd, FromRawFd, OwnedFd}, path::PathBuf, process::{Command, Stdio}};

use datarace_plugin_api::wrappers::PluginHandle;
use proton_finder::GameDrive;

use crate::data::PageTelemetry;

const BRIDGE_EXE_NAME:&'static str = "shm-bridge-rf2.exe";
const GAME_ID:u32 = 365960;

// In case you are curious, YES, those $ marks are really in the memory map path
// I could ask him why he did this, it causes hell when passed through cli,
// and looks like late a 2000s xXx_gamertag_xXx, but whatever...

/// 50 fps
const MM_TELEMETRY_FILE_NAME:&'static str = "$rFactor2SMMP_Telemetry$";
/// 5 fps
const MM_SCORING_FILE_NAME:&'static str = "$rFactor2SMMP_Scoring$";
/// 3 fps
const MM_RULES_FILE_NAME:&'static str = "$rFactor2SMMP_Rules$";
/// Once per session
const MM_MULIT_RULES_FILE_NAME:&'static str = "$rFactor2SMMP_MultiRules$";
/// 400 fps
const MM_FORCE_FEEDBACK_FILE_NAME:&'static str = "$rFactor2SMMP_ForceFeedback$";
/// 400 fps, default unsubscribed
const MM_GRAPHICS_FILE_NAME:&'static str = "$rFactor2SMMP_Graphics$";
/// 100 fps
const MM_PITINFO_FILE_NAME:&'static str = "$rFactor2SMMP_PitInfo$";
/// 1 fps, default unsubscribed
const MM_WEATHER_FILE_NAME:&'static str = "$rFactor2SMMP_Weather$";
/// 5 fps (plus on tracked callback from the game)
const MM_EXTENDED_FILE_NAME:&'static str = "$rFactor2SMMP_Extended$";

/// Checks if requirements are met
/// If not install the software
pub(crate) fn init_setup(handle: &PluginHandle) -> Result<(), String> {
    let prefix = match proton_finder::get_game_drive(GAME_ID) {
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
        return Err("bridge missing".to_string());
    }

    
    // TODO check if bridge is already running
    // Check if the game is running
    

    // TODO install rF2SharedMemoryMap Plugin if not present
    // And make sure it is enabled, for which we need to parse
    // UserData/player/CustomPluginVariables.json

    // TODO check protontricks exists

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

pub(crate) struct GameRunningHelperState {
    running: Option<sysinfo::Pid>,
    bridge: Option<sysinfo::Pid>,
    bridge_path: PathBuf
}

impl GameRunningHelperState {
    pub(crate) fn new(handle: &PluginHandle) -> Option<Self> {
        // Sysinfo keeps the files open, and we kind of don't want that
        sysinfo::set_open_files_limit(0);

        let prefix = match proton_finder::get_game_drive(GAME_ID) {
            Ok(res) => res,
            Err(res) => {
                handle.log_error("$STEAM_DIR was set, but no steam install found there!");
                res
            }
        }?; // Yes, this doesn't log proper errors, but if this fails the plugin would have failed
        // init with error messages
        let path = shm_bridge_path(&prefix)?;

        if !path.exists() {
            return None;
        }

        Some(GameRunningHelperState {
            running: None,
            bridge: None,
            bridge_path: path
        })
    }
}

/// This checks if a certain process (with a certain cmdline) is running, and retrieves the pid.
/// Passing in the previous PID will cut down on having to load and search through all processes.
/// But in case this process is no longer running or a different process, it will do a full check
/// for another process.
/// PID is determined on with find and contains on the cmdline, meaning if multiple processes are
/// running the same (or similar) then the first PID is grabbed
fn check_for_program_running(pid: Option<sysinfo::Pid>, name_frag: String) -> Option<sysinfo::Pid> {
    if let Some(pid) = &pid {
        // We know a pid which it ran with, so we just gather info on this one
        let mut s = sysinfo::System::new();
        s.refresh_pids_specifics(&[*pid], 
            sysinfo::ProcessRefreshKind::new()
                .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
        );

        if let Some(pro) = s.process(*pid) {
            if cmd_combiner(pro.cmd()).contains(&name_frag) {
                return Some(pro.pid());
            } else {
                // There is a foreign process running under our ID, we retry
                return check_for_program_running(None, name_frag);
            }

        } else {
            // Our process is gone, but don't worry, maybe he is still out there under another pid
            return check_for_program_running(None, name_frag);
        }
    } else {
        // We scan all processes
        let mut s = sysinfo::System::new();
        s.refresh_processes_specifics(
            sysinfo::ProcessRefreshKind::new()
                .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
        );

        if let Some(pro) = s.processes().values().find(|val| {
            // println!("Here with {}: {}", val.pid().to_string(), val.cmd());
            cmd_combiner(val.cmd()).contains(&name_frag)
            
        }) {
            return Some(pro.pid());
        }
    }
    
    None
}

fn cmd_combiner(cmd: &[String]) -> String {
    let mut output = String::new();

    for item in cmd {
        output = format!("{output} {item}");
    }

    output
}

/// Checks if the game is running
pub(crate) fn check_if_game_running(helper_state: &mut GameRunningHelperState) -> bool {
    // We find the stable entry process which cmdline looks something like this:
    // Z:\home\Lukas\.local\share\Steam\steamapps\common\Assetto Corsa Competizione\acc.exe
    // let rf2_bin_name = "rFactor 2/Launcher/Launch rFactor.exe".to_string();
    let rf2_bin_name = "rFactor 2\\Bin64\\rFactor2.exe".to_string();
     

    helper_state.running = check_for_program_running(helper_state.running, rf2_bin_name);
    helper_state.running.is_some()
}

fn check_for_bridge(helper_state: &mut GameRunningHelperState) -> bool {
    helper_state.bridge = check_for_program_running(helper_state.bridge, format!("DataRace\\{}", BRIDGE_EXE_NAME));
    
    helper_state.bridge.is_some()
}

pub(crate) fn connect(handle: &PluginHandle, helper_state: &mut GameRunningHelperState) -> Result<MapHolder, String> {

    if !check_for_bridge(helper_state) {
        handle.log_info("bridge was not running, launching bridge");

        // Spawning a new bridge process
        let res = Command::new("protontricks-launch")
            .arg("--appid")
            .arg(GAME_ID.to_string())
            .arg(helper_state.bridge_path.as_os_str())
            
            .arg("--map")
            .arg(MM_TELEMETRY_FILE_NAME)


            .arg("--size")
            .arg(size_of::<PageTelemetry>().to_string())

            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())

            
            .spawn();

        match res {
            Ok(_) => (),
            Err(e) => {
                helper_state.bridge = None;
                return Err(format!("Failed to launch shm-bridge: {}", e.to_string()))
            }
        };

        // Let the proton and the programm spinup
        std::thread::sleep(std::time::Duration::from_secs(5));

        if !check_for_bridge(helper_state) {
            return Err("Failed to launch shm-bridge: Crashed on startup".to_string());
        }
    }

    // Mounting the memory maps
    Ok(MapHolder {
        telemetry: SharedMemory::<PageTelemetry>::connect(MM_TELEMETRY_FILE_NAME)?
    })
}

pub(crate) fn disconnect(handle: &PluginHandle, helper_state: &mut GameRunningHelperState, holder: Option<MapHolder>) {
    drop(holder); // disconnect the memory maps

    // As the bridge has to be running before the game is launched, if the game is still
    // running we won't take down the bridge
    if !check_if_game_running(helper_state) {
        if let Some(pid) = helper_state.bridge {
            let mut s = sysinfo::System::new();    
            s.refresh_pids_specifics(&[pid], 
                sysinfo::ProcessRefreshKind::new()
                    .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
            );

            if let Some(pro) = s.process(pid) {
                pro.kill_with(sysinfo::Signal::Interrupt);
                // pro.kill();
                
                std::thread::sleep(std::time::Duration::from_secs(2));
                if check_for_bridge(helper_state) {
                    handle.log_error("Failed to shutdown bridge, kill it manually!");
                }
                
                // TODO shm-bridge clean up run
            } else {
                helper_state.bridge = None;
            }
        }
    } else if helper_state.bridge.is_some() {
        handle.log_info("Updater Disconnecting while game still running, shm-bridge not shutdown")
    }
}

/// Holds all the memory maps
pub struct MapHolder {
    pub telemetry: SharedMemory<PageTelemetry>
}

// Simetry
// MIT License
//
// Copyright (c) 2023 Adnan Ademovic
// Copyright (c) 2024 Damir Jelić
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
pub struct SharedMemory<T> {
    _fd: OwnedFd,
    memory: *mut c_void,
    phantom_data: PhantomData<T>,
}

// Send + Sync here is fine because we own the file descriptor and pointer to the mmapped region
// and we're only reading from it.
unsafe impl<T: Send + std::fmt::Debug> Send for SharedMemory<T> {}
unsafe impl<T: Sync + std::fmt::Debug> Sync for SharedMemory<T> {}

impl<T> SharedMemory<T> {
    fn connect(foo: &str) -> Result<Self, String> {
        let path = CString::new(format!("/{foo}")).expect("We should be able to build this static C string");

        let fd = unsafe { libc::shm_open(path.as_ptr(), libc::SHM_RDONLY, 0) };

        if fd == -1 {
            Err(format!("Opening the {} file failed: {}", path.to_string_lossy(), std::io::Error::last_os_error().to_string()))
        } else {
            let len = std::mem::size_of::<T>();
            let fd = unsafe { OwnedFd::from_raw_fd(fd) };

            let memory = unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    len,
                    libc::PROT_READ,
                    libc::MAP_SHARED,
                    fd.as_raw_fd(),
                    0,
                )
            };

            if memory == libc::MAP_FAILED {
                Err(format!("Unable to mmap the opened SHM file {}: {}", path.to_string_lossy(), std::io::Error::last_os_error().to_string()))
            } else {
                Ok(Self {
                    _fd: fd,
                    memory,
                    phantom_data: Default::default(),
                })
            }
        }
    }

    /// This gives you a reference into the struct laying in the memory map
    /// As such, against Rust guidelines, while holding this immutable reference
    /// the data can be modified by the game, so it is a good idea to clone
    pub fn get(&self) -> &T {
        unsafe { &(*(self.memory as *const T)) }
    }
}
