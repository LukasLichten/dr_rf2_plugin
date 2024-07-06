use datarace_plugin_api::{macros::generate_property_handle, wrappers::{DataStoreReturnCode, PluginHandle, Property, PropertyHandle}};

use crate::{share::{self, check_if_game_running}, MapHolder};

const P_EXTRA: PropertyHandle = generate_property_handle!("rF2-Reader.extra");
const P_TELEMETRY_UPDATE: PropertyHandle = generate_property_handle!("rF2-Reader.telemetry.update");

/// Creates the property handles during init
pub(crate) fn init_properties(handle: &PluginHandle) -> Result<(), String> {
    create_prop(&handle, "extra", P_EXTRA, Property::None)?;
    
    create_prop(&handle, "telemetry.update", P_TELEMETRY_UPDATE, Property::Int(0))?;

    Ok(())
}

/// Turns initializing a property into a oneliner
fn create_prop(handle: &PluginHandle, prop_name: &str, prop_handle: PropertyHandle, init_value: Property) -> Result<(),String> {
    // We use this helper so I can forward errors on property creation
    // And keep creation of a property single line
    match handle.create_property(prop_name, prop_handle, init_value) {
        DataStoreReturnCode::Ok => Ok(()),
        e => Err(e.to_string())
    }
}

pub(crate) struct ReaderState {
    telemetry_update_version: u32,
    version_last_increment: Option<std::time::Instant>,
}

impl Default for ReaderState {
    fn default() -> Self {
        ReaderState {
            telemetry_update_version: 0,
            version_last_increment: None,
        }
    }
}

/// Reads memory map
/// Ok(game running), if in doubt return false
pub(crate) fn update_properties(handle: &PluginHandle, mount: &MapHolder, state: &mut ReaderState, runner_checkgame_state: &mut share::GameRunningHelperState) -> Result<bool, String> {
    if state.telemetry_update_version != mount.telemetry.get().header.version_update_begin {
        state.version_last_increment = None;

        let update = mount.telemetry.get().clone();

        // We clone, so read the entire memory map, to insure a none torne frame
        // we check the begin and end version, and only update if they match
        if update.header.version_update_begin == update.header.version_update_end {
            state.telemetry_update_version = update.header.version_update_begin;

            handle.update_property(P_TELEMETRY_UPDATE, Property::Int(update.header.version_update_begin as i64));
        }
    } else if state.version_last_increment.is_none() {
        state.version_last_increment = Some(std::time::Instant::now());
    }
    

    // Triggering game running check due to lack of updates 
    if let Some(last) = state.version_last_increment {
        let now = std::time::Instant::now();
        if now > (last + std::time::Duration::from_secs(5)) {
            // handle.log_info("Time is up!");
            if check_if_game_running(runner_checkgame_state) {
                // Game still running, reset timer
                state.version_last_increment = Some(now);
            } else {
                return Ok(false);
            }
        }
    }

    Ok(true)
}
