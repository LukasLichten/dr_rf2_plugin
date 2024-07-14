use std::sync::atomic::{AtomicU32, Ordering};

use datarace_plugin_api::{macros::{get_state, save_state_now}, wrappers::{Message, PluginHandle}};

pub(crate) type PluginState = State;

// This is requires to handle deallocating strings
datarace_plugin_api::macros::free_string_fn!();

// Generates the required plugin description
// You have to pass in literals (at least so far, unfortunatly)
datarace_plugin_api::macros::plugin_descriptor_fn!("rF2-Reader", 0, 1, 0);

/// Mounts the memory maps
#[allow(dead_code)]
mod share;
/// Stores the game structs from the memory maps
#[allow(dead_code)]
pub(crate) mod data;
/// Contains the propertyhandles and does the writing to them
mod reader;

pub use share::{MapHolder, SharedMemory};

pub(crate) struct State {
    // Used to lock the update thread
    // 0 unlocked
    // 1 is lock requested
    // 2 is locked
    // 3 plugin shutdown requested
    // 4 plugin shutdown
    // 100 updater offline
    // 101 updater offline and locked
    update_lock: AtomicU32,
}

#[datarace_plugin_api::macros::plugin_init]
fn handle_init(handle: PluginHandle) -> Result<(),String> {
    // reader::init_properties(&handle)?; // Testing

    // Installation of memory map bridge (on linux) and plugin
    share::init_setup(&handle)?;

    // Property creation
    reader::init_properties(&handle)?;

    // State
    let state = State { update_lock: AtomicU32::new(100) };
    unsafe { save_state_now!(handle, state) };



    // Ok(state)
    Ok(())
}

#[datarace_plugin_api::macros::plugin_update]
fn handle_update(handle: PluginHandle, msg: Message) -> Result<(), String> {
    let state = get_state!(handle).ok_or("Unable to aquire plugin state")?;

    handle.log_info(state.update_lock.load(Ordering::Acquire));

    match msg {
        Message::StartupFinished => {
            handle.log_info("Startup completed, starting background worker thread");
            std::thread::spawn(|| updater(handle));
        },
        Message::Lock => {
            // Handling the lock
            match state.update_lock.load(Ordering::Acquire) {
                2 => (), // already locked
                100 => {
                    // Preventing launching of the update thread
                    if let Err(_) = state.update_lock.compare_exchange(100, 101, Ordering::AcqRel, Ordering::Acquire) {
                        // Launched anyway, so we now go stop the running thread
                        return handle_update(handle, msg); 
                    }
                },
                101 => (), // not started, but locked
                _ =>  {
                    state.update_lock.store(1, Ordering::Release); // Sending request to lock
                    while state.update_lock.load(Ordering::Acquire) == 1 {
                        atomic_wait::wait(&state.update_lock, 1);
                    }
                    // Process locked
                }
            }
            

        },
        Message::Unlock => {
            // Handling the unlock
            match state.update_lock.load(Ordering::Acquire) {
                2 | 10 => {
                    state.update_lock.store(0, Ordering::Release);
                    atomic_wait::wake_all(&state.update_lock);
                },
                101 => {
                    state.update_lock.store(100, Ordering::Release);
                    atomic_wait::wake_all(&state.update_lock); // We wake here to allow locked
                    // startups to escape
                },
                _ => ()
            }
            

        },
        Message::Shutdown => {
            // Shutting down the update thread
            if state.update_lock.swap(3, Ordering::AcqRel) < 100 {
                while state.update_lock.load(Ordering::Acquire) == 3 {
                    atomic_wait::wait(&state.update_lock, 3);
                }
            }

            handle.log_info("Good Night!");
            unsafe { datarace_plugin_api::macros::drop_state_now!(handle) }
        },
        Message::InternalMsg(num) => {
            match num {
                -1 => {
                    // TODO implement graceful shutdown option once available

                    
                    unsafe { datarace_plugin_api::macros::drop_state_now!(handle) }
                    return Err("Background thread shutdown, crashing the plugin!".to_string());
                },
                _ => ()
            }
        },
        Message::OtherPluginStarted(_) => (),
        _ => {
            handle.log_error("Unkown Message received (update this plugin)");
        }
    }

    Ok(())
}

/// Contains the passive part, checking if the game is running, and launching the active part
fn updater(handle: PluginHandle) {
    let sta = get_state!(handle).expect("Gimme!");


    let mut runchecker_helper_state = if let Some(res) = share::GameRunningHelperState::new(&handle) {
        res
    } else {
        handle.log_error("Updater aborting due to being unable to aquire necessary resources!");
        handle.send_internal_msg(-1);
        // TODO implement a way to shutdown the plugin
        return;
    };



    // Outer game check running loop
    loop {

        if share::check_if_game_running(&mut runchecker_helper_state) {
            handle.log_info("Game is detected running, starting updater...");

            match share::connect(&handle, &mut runchecker_helper_state) {
                Ok(mount) => {
                    let exit = !runner_loop(sta, &handle, &mount, &mut runchecker_helper_state);
                    handle.log_info("Exiting Updater...");
                    share::disconnect(&handle, &mut runchecker_helper_state, Some(mount));

                    if exit {
                        return;
                    }
                },
                Err(e) => {
                    share::disconnect(&handle, &mut runchecker_helper_state, None);
                    handle.log_error(format!("Updater failed to mount memory maps (Retrying): {e}"));
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(5));

    }
}

/// Contains to active update runner and it's locking mechanism
/// return value indicates if programm should exit (false), or continue (true)
fn runner_loop(sta: &PluginState, handle: &PluginHandle, mount: &MapHolder, runchecker_helper_state: &mut share::GameRunningHelperState) -> bool {
    // Game running, starting up loop
    // Checking for startup handle lock
    while let Err(v) = sta.update_lock.compare_exchange(100, 0, Ordering::AcqRel, Ordering::Acquire) {
        match v {
            101 => atomic_wait::wait(&sta.update_lock, v),
            3 => return false,
            v => panic!("Updater unknown lock state {v} abort!")
        };
    }
    handle.log_info("Updater Started");

    let mut reader_state = reader::ReaderState::default();

    loop {
        match sta.update_lock.load(Ordering::Acquire) {
            1 | 101 => {
                sta.update_lock.store(2, Ordering::Release);
                atomic_wait::wake_all(&sta.update_lock);

                handle.log_info("Updater: Locking Down!");
                while sta.update_lock.load(Ordering::Acquire) == 2 {
                    atomic_wait::wait(&sta.update_lock, 2);
                }
                handle.log_info("Updater: Unlocked");
            },
            3 => {
                handle.log_info("Updater: Shutdown");
                sta.update_lock.store(4, Ordering::Release);
                atomic_wait::wake_all(&sta.update_lock);
                return false;
            },
            _ => ()
        }

        // Actual work
        match reader::update_properties(handle, mount, &mut reader_state, runchecker_helper_state) {
            Ok(true) => (),
            Ok(false) => break,
            Err(e) => {
                handle.log_error(format!("Reading update failed: {e}"));
            }
        }
    }

    // handle.log_info("Hewo!");

    match sta.update_lock.swap(100, Ordering::AcqRel) {
        1 | 101 => {
            sta.update_lock.store(101, Ordering::Release);
            atomic_wait::wake_all(&sta.update_lock);
            true
        },
        3 => {
            sta.update_lock.store(4, Ordering::Release);
            atomic_wait::wake_all(&sta.update_lock);
            false
        },
        _ => true
    }
}
