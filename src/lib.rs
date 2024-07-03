use std::sync::atomic::{AtomicU32, Ordering};

use datarace_plugin_api::{macros::{get_state, save_state_now}, wrappers::{Message, PluginHandle}};

pub(crate) type PluginState = State;

// This is requires to handle deallocating strings
datarace_plugin_api::macros::free_string_fn!();

// Generates the required plugin description
// You have to pass in literals (at least so far, unfortunatly)
datarace_plugin_api::macros::plugin_descriptor_fn!("rF2-Reader", 0, 1, 0);

/// Mounts the memory maps
mod share;
/// Stores the game structs from the memory maps
pub(crate) mod data;
/// Contains the propertyhandles and does the writing to them
mod reader;

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

// This is temporary
struct Helper {
    handle: PluginHandle
}

unsafe impl Sync for Helper {}
unsafe impl Send for Helper {}

#[datarace_plugin_api::macros::plugin_init]
fn handle_init(handle: PluginHandle) -> Result<(),String> {
    // Installation of memory map bridge (on linux) and plugin
    share::init_setup(&handle)?;

    // Property creation
    reader::init_properties(&handle)?;

    // State
    let state = State { update_lock: AtomicU32::new(100) };
    unsafe { save_state_now!(handle, state) };

    let h = Helper { handle };
    std::thread::spawn(|| updater(h));


    // Ok(state)
    Ok(())
}

#[datarace_plugin_api::macros::plugin_update]
fn handle_update(handle: PluginHandle, msg: Message) -> Result<(), String> {
    let state = get_state!(handle).ok_or("Unable to aquire plugin state")?;

    handle.log_info(state.update_lock.load(Ordering::Acquire));

    match msg {
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
        Message::Unknown => {
            handle.log_error("Unkown Message received (update this plugin)");
        }
    }

    Ok(())
}

fn updater(handle: Helper) {
    let handle = handle.handle;
    
    let sta = get_state!(handle).expect("Gimme!");
    // Checking for startup handle lock
    while let Err(v) = sta.update_lock.compare_exchange(100, 0, Ordering::AcqRel, Ordering::Acquire) {
        match v {
            101 => atomic_wait::wait(&sta.update_lock, v),
            3 => return,
            v => panic!("Updater unknown lock state {v} abort!")
        };
    }
    handle.log_info("Updater Started");

    let mut i:usize = 0;

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
                return;
            },
            9000 => {
                // Placeholder for exit condition
                break;
            },
            _ => ()
        }

        // Idk, do something
        i = i.wrapping_add(1);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    match sta.update_lock.swap(100, Ordering::AcqRel) {
        1 | 101 => {
            sta.update_lock.store(101, Ordering::Release);
            atomic_wait::wake_all(&sta.update_lock);
        },
        3 => {
            sta.update_lock.store(4, Ordering::Release);
            atomic_wait::wake_all(&sta.update_lock);
            return;
        },
        _ => ()
    }
}
