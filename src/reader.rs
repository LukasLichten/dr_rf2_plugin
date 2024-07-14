use std::usize;

use datarace_plugin_api::{macros::generate_property_handle, wrappers::{DataStoreReturnCode, PluginHandle, Property, PropertyHandle}};

use crate::{data::{PageScoring, PageVehicleTelemetry, RF2IgnitionStarterStatus, RF2RearFlapLegalStatus, MAX_MAPPED_VEHICLES}, share::{self, check_if_game_running}, MapHolder};

const P_EXTRA: PropertyHandle = generate_property_handle!("rF2-Reader.extra");

// Telemetry
const P_TELEMETRY_UPDATE: PropertyHandle = generate_property_handle!("rF2-Reader.telemetry.update");
const P_DEBUG_TELEMETRY_TIME: PropertyHandle = generate_property_handle!("rf2-reader.debug.telemetry.time");

// Left overs:
// /// slot ID (note that it can be re-used in multiplayer after someone leaves)    
// pub id: i32,
// /// time since last update (seconds)    
// pub delta_time: f64,
// /// velocity (meters/sec) in local vehicle coordinates    
// pub local_vel: PageVec3,
// /// acceleration (meters/sec^2) in local vehicle coordinates    
// pub local_accel: PageVec3,
//
// // Orientation and derivatives
// /// rows of orientation matrix (use TelemQuat conversions if desired), also converts local    
// pub ori: [PageVec3; 3],
// // vehicle vectors into world X, Y, or Z using dot product of rows 0, 1, or 2 respectively
// /// rotation (radians/sec) in local vehicle coordinates    
// pub local_rot: PageVec3,
// /// rotational acceleration (radians/sec^2) in local vehicle coordinates    
// pub local_rot_accel: PageVec3,
//
// /// whether any parts (besides wheels) have been detached    
// pub detached: u8,
//     /// dent severity at 8 locations around the car (0=none, 1=some, 2=more)    
//     pub dent_severity: [u8; 8],
//     /// time of last impact    
//     pub last_impact_et: f64,
//     /// magnitude of last impact    
//     pub last_impact_magnitude: f64,
//     /// location of last impact    
//     pub last_impact_pos: PageVec3,
//     /// offset from static CG to graphical center    
//     pub physics_to_graphics_offset: [f32; 3],
//
//     // Future use (contains now electric drivetrain info)
//     /// for future use (note that the slot ID has been moved to mID above)    
//     expansion: [Garbage; 152],

const P_TELEMETRY_SESSION_ELAPSED_TIME: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.session_elapsed_time");
const P_TELEMETRY_LAP_NUMBER: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.lap_number");
const P_TELEMETRY_LAP_ELAPSED_TIME: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.lap_elapsed_time");
const P_TELEMETRY_VEHICLE_NAME: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.vehicle_name");
const P_TELEMETRY_TRACK_NAME: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.track_name");

const P_TELEMETRY_POS_X: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.pos_x");
const P_TELEMETRY_POS_Y: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.pos_y");
const P_TELEMETRY_POS_Z: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.pos_z");

const P_TELEMETRY_GEAR: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.gear");
const P_TELEMETRY_ENGINE_RPM: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.engine.rpm");
const P_TELEMETRY_ENGINE_WATER_TEMP: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.engine.water_temp");
const P_TELEMETRY_ENGINE_OIL_TEMP: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.engine.oil_temp");
const P_TELEMETRY_CLUTCH_RPM: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.clutch_rpm");

const P_TELEMETRY_THROTTLE_RAW: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.throttle_raw");
const P_TELEMETRY_BRAKE_RAW: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.brake_raw");
const P_TELEMETRY_CLUTCH_RAW: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.clutch_raw");
const P_TELEMETRY_STEERING_RAW: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.steering_raw");

const P_TELEMETRY_THROTTLE_FILTERED: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.throttle_filtered");
const P_TELEMETRY_BRAKE_FILTERED: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.brake_filtered");
const P_TELEMETRY_CLUTCH_FILTERED: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.clutch_filtered");
const P_TELEMETRY_STEERING_FILTERED: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.steering_filtered");

const P_TELEMETRY_STEERING_SHAFT_TORQUE: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.steering_shaft_torque");
const P_TELEMETRY_FRONT_3RD_SPRING_DEFLECTION: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.front.3rd_spring_deflection");
const P_TELEMETRY_REAR_3RD_SPRING_DEFLECTION: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.rear.3rd_spring_deflection");

const P_TELEMETRY_FRONT_WING_HEIGHT: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.front.wing_height");
const P_TELEMETRY_FRONT_RIDE_HEIGHT: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.front.ride_height");
const P_TELEMETRY_REAR_RIDE_HEIGHT: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.rear.ride_height");
const P_TELEMETRY_DRAG: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.drag");
const P_TELEMETRY_FRONT_DOWNFORCE: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.front.downforce");
const P_TELEMETRY_REAR_DOWNFORCE: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.rear.downforce");

const P_TELEMETRY_FUEL: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.fuel");
const P_TELEMETRY_ENGINE_MAX_RPM: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.engine.max_rpm");
const P_TELEMETRY_PIT_SCHEDULED_STOPS: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.pit.scheduled_stops");
const P_TELEMETRY_ENGINE_OVERHEATING: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.engine.overheating");
const P_TELEMETRY_HEADLIGHTS: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.headlights");

const P_TELEMETRY_ENGINE_TORQUE: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.engine.torque");
const P_TELEMETRY_CURRENT_SECTOR: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.current_sector");
const P_TELEMETRY_SPEED_LIMITER: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.speed_limiter");
const P_TELEMETRY_MAX_GEARS: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.max_gears");
const P_TELEMETRY_FRONT_TIRE_COMPOUND_INDEX: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.front.tire_compound_index");
const P_TELEMETRY_REAR_TIRE_COMPOUND_INDEX: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.rear.tire_compound_index");
const P_TELEMETRY_FUEL_CAPACITY: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.fuel_capacity");
const P_TELEMETRY_FRONT_FLAP_ACTIVATED: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.front.flap_activated");
const P_TELEMETRY_REAR_FLAP_ACTIVATED: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.rear.flap_activated");
const P_TELEMETRY_REAR_FLAP_DETECTED: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.rear.flap_detected");
const P_TELEMETRY_REAR_FLAP_ALLOWED: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.rear.flap_allowed");
const P_TELEMETRY_ENGINE_IGNITION: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.engine.ignition");
const P_TELEMETRY_ENGINE_STARTER: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.engine.starter");

const P_TELEMETRY_FRONT_TIRE_COMPOUND_NAME: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.front.tire_compound_name");
const P_TELEMETRY_REAR_TIRE_COMPOUND_NAME: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.rear.tire_compound_name");
const P_TELEMETRY_SPEED_LIMITER_AVAILABLE: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.speed_limiter_available");
const P_TELEMETRY_ANTI_STALL_ACTIVATED: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.anti_stall_activated");
const P_TELEMETRY_VISIUAL_STEERING_WHEEL_RANGE: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.visual_steering_wheel_range");
const P_TELEMETRY_FRONT_BRAKE_BIAS: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.front.brake_bias");
const P_TELEMETRY_REAR_BRAKE_BIAS: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.rear.brake_bias");
const P_TELEMETRY_ENGINE_TURBO_BOOST_PRESSURE: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.engine.turbo_boost_pressure");
const P_TELEMETRY_PHYSICAL_WHEEL_RANGE: PropertyHandle = generate_property_handle!("rf2-reader.telemetry.physical_steering_wheel_range");

// TODO wheels

//
//     // keeping this at the end of the structure to make it easier to replace in future versions
//     /// wheel info (front left, front right, rear left, rear right)    
//     pub wheels: [PageWheelTelemetry; 4],
// }
//
// #[repr(C, packed(4))]
// #[derive(Copy, Clone, Debug)]
// pub struct PageWheelTelemetry {
//     /// meters
//     pub suspension_deflection: f64,
//     /// meters
//     pub ride_height: f64,
//     /// pushrod load in Newtons
//     pub susp_force: f64,
//     /// Celsius
//     pub brake_temp: f64,
//     /// currently 0.0-1.0, depending on driver input and brake balance; will convert to true brake pressure (kPa) in future
//     pub brake_pressure: f64,
//
//     /// radians/sec
//     pub rotation: f64,
//     /// lateral velocity at contact patch
//     pub lateral_patch_vel: f64,
//     /// longitudinal velocity at contact patch
//     pub longitudinal_patch_vel: f64,
//     /// lateral velocity at contact patch
//     pub lateral_ground_vel: f64,
//     /// longitudinal velocity at contact patch
//     pub longitudinal_ground_vel: f64,
//     /// radians (positive is left for left-side wheels, right for right-side wheels)
//     pub camber: f64,
//     /// Newtons
//     pub lateral_force: f64,
//     /// Newtons
//     pub longitudinal_force: f64,
//     /// Newtons
//     pub tire_load: f64,
//
//     /// an approximation of what fraction of the contact patch is sliding
//     pub grip_fract: f64,
//     /// kPa (tire pressure)
//     pub pressure: f64,
//     /// Kelvin (subtract 273.15 to get Celsius), left/center/right (not to be confused with inside/center/outside!)
//     pub temperature: [f64; 3],
//     /// wear (0.0-1.0, fraction of maximum) ... this is not necessarily proportional with grip loss
//     pub wear: f64,
//     /// the material prefixes from the TDF file
//     pub terrain_name: String16,
//     /// Enum for surface type
//     pub surface_type: u8,
//     /// whether tire is flat
//     pub flat: u8,
//     /// whether wheel is detached
//     pub detached: u8,
//     /// tire radius in centimeters
//     pub static_undeflected_radius: u8,
//
//     /// how much is tire deflected from its (speed-sensitive) radius
//     pub vertical_tire_deflection: f64,
//     /// wheel's y location relative to vehicle y location
//     pub wheel_ylocation: f64,
//     /// current toe angle w.r.t. the vehicle
//     pub toe: f64,
//
//     /// rough average of temperature samples from carcass (Kelvin)
//     pub tire_carcass_temperature: f64,
//     /// rough average of temperature samples from innermost layer of rubber (before carcass) (Kelvin)
//     pub tire_inner_layer_temperature: [f64; 3],
//
//     /// for future use
//     expansion: [Garbage; 24],
// }
//

// Scoring
const P_SCORING_UPDATE: PropertyHandle = generate_property_handle!("rf2-reader.scoring.update");


/// Creates the property handles during init
pub(crate) fn init_properties(handle: &PluginHandle) -> Result<(), String> {
    create_prop(handle, "extra", P_EXTRA, Property::None)?;
    
    // Telemetry
    create_prop(handle, "telemetry.update", P_TELEMETRY_UPDATE, Property::Int(0))?;
    create_prop(handle, "debug.telemetry.time", P_DEBUG_TELEMETRY_TIME, Property::Duration(0))?;

    create_prop(handle, "telemetry.session_elapsed_time", P_TELEMETRY_SESSION_ELAPSED_TIME, Property::Duration(0))?;
    create_prop(handle, "telemetry.lap_number", P_TELEMETRY_LAP_NUMBER, Property::Int(-1))?;
    create_prop(handle, "telemetry.lap_elapsed_time", P_TELEMETRY_LAP_ELAPSED_TIME, Property::Duration(0))?;
    create_prop(handle, "telemetry.track_name", P_TELEMETRY_TRACK_NAME, Property::from_string(""))?;
    create_prop(handle, "telemetry.vehicle_name", P_TELEMETRY_VEHICLE_NAME, Property::from_string(""))?;

    create_prop(handle, "telemetry.pos_x", P_TELEMETRY_POS_X, Property::Float(0.0))?;
    create_prop(handle, "telemetry.pos_y", P_TELEMETRY_POS_Y, Property::Float(0.0))?;
    create_prop(handle, "telemetry.pos_z", P_TELEMETRY_POS_Z, Property::Float(0.0))?;

    create_prop(handle, "telemetry.gear", P_TELEMETRY_GEAR, Property::Int(0))?;
    create_prop(handle, "telemetry.engine.rpm", P_TELEMETRY_ENGINE_RPM, Property::Float(0.0))?;
    create_prop(handle, "telemetry.engine.water_temp", P_TELEMETRY_ENGINE_WATER_TEMP, Property::Float(0.0))?;
    create_prop(handle, "telemetry.engine.oil_temp", P_TELEMETRY_ENGINE_OIL_TEMP, Property::Float(0.0))?;
    create_prop(handle, "telemetry.clutch_rpm", P_TELEMETRY_CLUTCH_RPM, Property::Float(0.0))?;

    create_prop(handle, "telemetry.throttle_raw", P_TELEMETRY_THROTTLE_RAW, Property::Float(0.0))?;
    create_prop(handle, "telemetry.brake_raw", P_TELEMETRY_BRAKE_RAW, Property::Float(0.0))?;
    create_prop(handle, "telemetry.clutch_raw", P_TELEMETRY_CLUTCH_RAW, Property::Float(0.0))?;
    create_prop(handle, "telemetry.steering_raw", P_TELEMETRY_STEERING_RAW, Property::Float(0.0))?;

    create_prop(handle, "telemetry.throttle_filtered", P_TELEMETRY_THROTTLE_FILTERED, Property::Float(0.0))?;
    create_prop(handle, "telemetry.brake_filtered", P_TELEMETRY_BRAKE_FILTERED, Property::Float(0.0))?;
    create_prop(handle, "telemetry.clutch_filtered", P_TELEMETRY_CLUTCH_FILTERED, Property::Float(0.0))?;
    create_prop(handle, "telemetry.steering_filtered", P_TELEMETRY_STEERING_FILTERED, Property::Float(0.0))?;

    create_prop(handle, "telemetry.steering_shaft_torque", P_TELEMETRY_STEERING_SHAFT_TORQUE, Property::Float(0.0))?;
    create_prop(handle, "telemetry.front.3rd_spring_deflection", P_TELEMETRY_FRONT_3RD_SPRING_DEFLECTION, Property::Float(0.0))?;
    create_prop(handle, "telemetry.rear.3rd_spring_deflection", P_TELEMETRY_REAR_3RD_SPRING_DEFLECTION, Property::Float(0.0))?;

    create_prop(handle, "telemetry.front.wing_height", P_TELEMETRY_FRONT_WING_HEIGHT, Property::Float(0.0))?;
    create_prop(handle, "telemetry.front.ride_height", P_TELEMETRY_FRONT_RIDE_HEIGHT, Property::Float(0.0))?;
    create_prop(handle, "telemetry.rear.ride_height", P_TELEMETRY_REAR_RIDE_HEIGHT, Property::Float(0.0))?;
    create_prop(handle, "telemetry.drag", P_TELEMETRY_DRAG, Property::Float(0.0))?;
    create_prop(handle, "telemetry.front.downforce", P_TELEMETRY_FRONT_DOWNFORCE, Property::Float(0.0))?;
    create_prop(handle, "telemetry.rear.downforce", P_TELEMETRY_REAR_DOWNFORCE, Property::Float(0.0))?;

    create_prop(handle, "telemetry.fuel", P_TELEMETRY_FUEL, Property::Float(0.0))?;
    create_prop(handle, "telemetry.engine.max_rpm", P_TELEMETRY_ENGINE_MAX_RPM, Property::Float(0.0))?;
    create_prop(handle, "telemetry.pit.scheduled_stops", P_TELEMETRY_PIT_SCHEDULED_STOPS, Property::Int(0))?;
    create_prop(handle, "telemetry.engine.overheating", P_TELEMETRY_ENGINE_OVERHEATING, Property::Bool(false))?;
    create_prop(handle, "telemetry.headlights", P_TELEMETRY_HEADLIGHTS, Property::Bool(false))?;
    
    create_prop(handle, "telemetry.engine.torque", P_TELEMETRY_ENGINE_TORQUE, Property::Float(0.0))?;
    create_prop(handle, "telemetry.current_sector", P_TELEMETRY_CURRENT_SECTOR, Property::Int(0))?;
    create_prop(handle, "telemetry.speed_limiter", P_TELEMETRY_SPEED_LIMITER, Property::Bool(false))?;
    create_prop(handle, "telemetry.max_gears", P_TELEMETRY_MAX_GEARS, Property::Int(0))?;
    create_prop(handle, "telemetry.front.tire_compound_index", P_TELEMETRY_FRONT_TIRE_COMPOUND_INDEX, Property::Int(0))?;
    create_prop(handle, "telemetry.rear.tire_compound_index", P_TELEMETRY_REAR_TIRE_COMPOUND_INDEX, Property::Int(0))?;
    create_prop(handle, "telemetry.fuel_capacity", P_TELEMETRY_FUEL_CAPACITY, Property::Float(0.0))?;
    create_prop(handle, "telemetry.front.flap_activated", P_TELEMETRY_FRONT_FLAP_ACTIVATED, Property::Bool(false))?;
    create_prop(handle, "telemetry.rear.flap_activated", P_TELEMETRY_REAR_FLAP_ACTIVATED, Property::Bool(false))?;
    create_prop(handle, "telemetry.rear.flap_detected", P_TELEMETRY_REAR_FLAP_DETECTED, Property::Bool(false))?;
    create_prop(handle, "telemetry.rear.flap_allowed", P_TELEMETRY_REAR_FLAP_ALLOWED, Property::Bool(false))?;
    create_prop(handle, "telemetry.engine.ignition", P_TELEMETRY_ENGINE_IGNITION, Property::Bool(false))?;
    create_prop(handle, "telemetry.engine.starter", P_TELEMETRY_ENGINE_STARTER, Property::Bool(false))?;

    create_prop(handle, "telemetry.front.tire_compound_name", P_TELEMETRY_FRONT_TIRE_COMPOUND_NAME, Property::from_string(""))?;
    create_prop(handle, "telemetry.rear.tire_compound_name", P_TELEMETRY_REAR_TIRE_COMPOUND_NAME, Property::from_string(""))?;
    create_prop(handle, "telemetry.speed_limiter_available", P_TELEMETRY_SPEED_LIMITER_AVAILABLE, Property::Bool(false))?;
    create_prop(handle, "telemetry.anti_stall_activated", P_TELEMETRY_ANTI_STALL_ACTIVATED, Property::Bool(false))?;
    create_prop(handle, "telemetry.visual_steering_wheel_range", P_TELEMETRY_VISIUAL_STEERING_WHEEL_RANGE, Property::Float(0.0))?;
    create_prop(handle, "telemetry.front.brake_bias", P_TELEMETRY_FRONT_BRAKE_BIAS, Property::Float(0.0))?;
    create_prop(handle, "telemetry.rear.brake_bias", P_TELEMETRY_REAR_BRAKE_BIAS, Property::Float(0.0))?;
    create_prop(handle, "telemetry.engine.turbo_boost_pressure", P_TELEMETRY_ENGINE_TURBO_BOOST_PRESSURE, Property::Float(0.0))?;
    create_prop(handle, "telemetry.physical_steering_wheel_range", P_TELEMETRY_PHYSICAL_WHEEL_RANGE, Property::Float(0.0))?;

    // Scoring
    create_prop(handle, "scoring.update", P_SCORING_UPDATE, Property::Int(0))?;

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
    telemetry_cache: TelemetryCache,

    scoring_update_version: u32,

    player_vehicle_id: i32,
    version_last_increment: Option<std::time::Instant>,
}

impl Default for ReaderState {
    fn default() -> Self {
        ReaderState {
            telemetry_update_version: 0,
            telemetry_cache: TelemetryCache {
                vehicle_name: String::new(),
                track_name: String::new(),
                front_tire_compound_name: String::new(),
                rear_tire_compound_name: String::new()
            },

            scoring_update_version: 0,
            player_vehicle_id: 0,
            version_last_increment: None,

        }
    }
}

/// Reads memory map
/// Ok(game running), if in doubt return false
pub(crate) fn update_properties(handle: &PluginHandle, mount: &MapHolder, state: &mut ReaderState, runner_checkgame_state: &mut share::GameRunningHelperState) -> Result<bool, String> {
    if state.scoring_update_version != mount.scoring.get().header.version_update_begin {
        let update = mount.scoring.get().clone();

        // We clone, so read the entire memory map, to insure a none torne frame
        // we check the begin and end version, and only update if they match
        if update.header.version_update_begin == update.header.version_update_end {
            state.scoring_update_version = update.header.version_update_begin;

            read_scoring(handle, update, state);

            handle.update_property(P_SCORING_UPDATE, Property::from(state.scoring_update_version));
        }
    }


    let telemetry_timing = std::time::Instant::now();
    if state.telemetry_update_version != mount.telemetry.get().header.version_update_begin {

        // Reference into memory that can be actively changed
        // Not great, but we hold this for a moment to find the player car
        // so we don't have to clone all Telemetry, just that of the player car
        let unstable = mount.telemetry.get();
        let begin = unstable.header.version_update_begin;
        let num_vehicles = if unstable.num_vehicles >= 0 && (unstable.num_vehicles as usize) <= MAX_MAPPED_VEHICLES {
            unstable.num_vehicles.clone() as usize
        } else {
            MAX_MAPPED_VEHICLES
        };

        let mut not_found = true;

        for i in 0..num_vehicles {
            let veh = unstable.vehicles[i];

            // Can we use it as an id to straight index? TODO
            if veh.id == state.player_vehicle_id {
                let update = veh.clone();

                // After we cloned only the vehicle we need we check the update id for change
                // we check the begin and end version, and only update if they match
                if begin == mount.telemetry.get().header.version_update_end {
                    state.telemetry_update_version = begin;

                    read_telemetry(handle, update, &mut state.telemetry_cache);

                    handle.update_property(P_TELEMETRY_UPDATE, Property::from(begin));
                    handle.update_property(P_DEBUG_TELEMETRY_TIME, Property::from(std::time::Instant::now() - telemetry_timing));

                    // Reason we are doing this is to prevent a torn frame from deadlocking us
                    not_found = false;
                }

                break;
            }
        }

        if not_found && state.version_last_increment.is_none() {
            state.version_last_increment = Some(std::time::Instant::now());
        } else if !not_found {
            state.version_last_increment = None;
        }

    } else if state.version_last_increment.is_none() {
        state.version_last_increment = Some(std::time::Instant::now());
    }

    // Graphics contains the car the player is currently spectating,
    // but graphics is also not subscribed by default
    // *And the implementation in data.rs is lacking*
    

    // Triggering game running check due to lack of updates 
    if let Some(last) = state.version_last_increment {
        let now = std::time::Instant::now();
        if now > (last + std::time::Duration::from_secs(5)) {
            // handle.log_info("Time is up!");
            if check_if_game_running(runner_checkgame_state) {
                // Game still running, reset timer
                // handle.log_info("Game still running");
                state.version_last_increment = Some(now);
            } else {
                // handle.log_info("Game no longer running");
                return Ok(false);
            }
        }
    }

    Ok(true)
}

struct TelemetryCache {
    vehicle_name: String,
    track_name: String,
    front_tire_compound_name: String,
    rear_tire_compound_name: String
}

fn read_telemetry(handle: &PluginHandle, update: PageVehicleTelemetry, cache: &mut TelemetryCache) {
    handle.update_property(P_TELEMETRY_SESSION_ELAPSED_TIME, Property::from_sec(update.elapsed_time));
    handle.update_property(P_TELEMETRY_LAP_NUMBER, Property::from(update.lap_number));
    handle.update_property(P_TELEMETRY_LAP_ELAPSED_TIME, Property::from_sec(update.elapsed_time - update.lap_start_et));
    help_read_string(handle, &update.vehicle_name, &mut cache.vehicle_name, P_TELEMETRY_VEHICLE_NAME);
    help_read_string(handle, &update.track_name, &mut cache.track_name, P_TELEMETRY_TRACK_NAME);

    
    handle.update_property(P_TELEMETRY_POS_X, Property::from(update.pos.x));
    handle.update_property(P_TELEMETRY_POS_Y, Property::from(update.pos.y));
    handle.update_property(P_TELEMETRY_POS_Z, Property::from(update.pos.z));
    

    handle.update_property(P_TELEMETRY_GEAR, Property::from(update.gear));
    handle.update_property(P_TELEMETRY_ENGINE_RPM, Property::from(update.engine_rpm));
    handle.update_property(P_TELEMETRY_ENGINE_WATER_TEMP, Property::from(update.engine_water_temp));
    handle.update_property(P_TELEMETRY_ENGINE_OIL_TEMP, Property::from(update.engine_oil_temp));
    handle.update_property(P_TELEMETRY_CLUTCH_RPM, Property::from(update.clutch_rpm));

    handle.update_property(P_TELEMETRY_THROTTLE_RAW, Property::from(update.unfiltered_throttle));
    handle.update_property(P_TELEMETRY_BRAKE_RAW, Property::from(update.unfiltered_brake));
    handle.update_property(P_TELEMETRY_CLUTCH_RAW, Property::from(update.unfiltered_clutch));
    handle.update_property(P_TELEMETRY_STEERING_RAW, Property::from(update.unfiltered_steering));

    handle.update_property(P_TELEMETRY_THROTTLE_FILTERED, Property::from(update.filtered_throttle));
    handle.update_property(P_TELEMETRY_BRAKE_FILTERED, Property::from(update.filtered_brake));
    handle.update_property(P_TELEMETRY_CLUTCH_FILTERED, Property::from(update.filtered_clutch));
    handle.update_property(P_TELEMETRY_STEERING_FILTERED, Property::from(update.filtered_steering));

    handle.update_property(P_TELEMETRY_STEERING_SHAFT_TORQUE, Property::from(update.steering_shaft_torque));
    handle.update_property(P_TELEMETRY_FRONT_3RD_SPRING_DEFLECTION, Property::from(update.front3rd_deflection));
    handle.update_property(P_TELEMETRY_REAR_3RD_SPRING_DEFLECTION, Property::from(update.rear3rd_deflection));

    handle.update_property(P_TELEMETRY_FRONT_WING_HEIGHT, Property::from(update.front_wing_height));
    handle.update_property(P_TELEMETRY_FRONT_RIDE_HEIGHT, Property::from(update.front_ride_height));
    handle.update_property(P_TELEMETRY_REAR_RIDE_HEIGHT, Property::from(update.rear_ride_height));
    handle.update_property(P_TELEMETRY_DRAG, Property::from(update.drag));
    handle.update_property(P_TELEMETRY_FRONT_DOWNFORCE, Property::from(update.front_downforce));
    handle.update_property(P_TELEMETRY_REAR_DOWNFORCE, Property::from(update.rear_downforce));

    handle.update_property(P_TELEMETRY_FUEL, Property::from(update.fuel));
    handle.update_property(P_TELEMETRY_ENGINE_MAX_RPM, Property::from(update.engine_max_rpm)); // infrequently
    handle.update_property(P_TELEMETRY_PIT_SCHEDULED_STOPS, Property::from(update.scheduled_stops)); // infrequently
    handle.update_property(P_TELEMETRY_ENGINE_OVERHEATING, Property::from(update.overheating != 0));
    handle.update_property(P_TELEMETRY_HEADLIGHTS, Property::from(update.headlights != 0));

    handle.update_property(P_TELEMETRY_ENGINE_TORQUE, Property::from(update.engine_torque));
    handle.update_property(P_TELEMETRY_CURRENT_SECTOR, Property::from(update.current_sector & 0x7FFFFFFF));
    handle.update_property(P_TELEMETRY_SPEED_LIMITER, Property::from(update.speed_limiter != 0));
    handle.update_property(P_TELEMETRY_MAX_GEARS, Property::from(update.max_gears)); // infrequently
    handle.update_property(P_TELEMETRY_FRONT_TIRE_COMPOUND_INDEX, Property::from(update.front_tire_compound_index)); // Slightly
    handle.update_property(P_TELEMETRY_REAR_TIRE_COMPOUND_INDEX, Property::from(update.rear_tire_compound_index)); // Slightly
    handle.update_property(P_TELEMETRY_FUEL_CAPACITY, Property::from(update.fuel_capacity)); // infrequently
    handle.update_property(P_TELEMETRY_FRONT_FLAP_ACTIVATED, Property::from(update.front_flap_activated != 0));
    handle.update_property(P_TELEMETRY_REAR_FLAP_ACTIVATED, Property::from(update.rear_flap_activated != 0));
    handle.update_property(P_TELEMETRY_REAR_FLAP_DETECTED, Property::from(update.rear_flap_legal_status == RF2RearFlapLegalStatus::DetectedButNotAllowedYet));
    handle.update_property(P_TELEMETRY_REAR_FLAP_ALLOWED, Property::from(update.rear_flap_legal_status == RF2RearFlapLegalStatus::Allowed));
    handle.update_property(P_TELEMETRY_ENGINE_IGNITION, Property::from(update.ignition_starter != RF2IgnitionStarterStatus::Off));
    handle.update_property(P_TELEMETRY_ENGINE_STARTER, Property::from(update.ignition_starter == RF2IgnitionStarterStatus::IgnitionAndStarter));

    help_read_string(handle, &update.front_tire_compound_name, &mut cache.front_tire_compound_name, P_TELEMETRY_FRONT_TIRE_COMPOUND_NAME);
    help_read_string(handle, &update.rear_tire_compound_name, &mut cache.rear_tire_compound_name, P_TELEMETRY_REAR_TIRE_COMPOUND_NAME);
    handle.update_property(P_TELEMETRY_SPEED_LIMITER_AVAILABLE, Property::from(update.speed_limiter_available != 0));
    handle.update_property(P_TELEMETRY_ANTI_STALL_ACTIVATED, Property::from(update.anti_stall_activated != 0));
    handle.update_property(P_TELEMETRY_VISIUAL_STEERING_WHEEL_RANGE, Property::from(update.visual_steering_wheel_range)); // infrequently
    handle.update_property(P_TELEMETRY_FRONT_BRAKE_BIAS, Property::from(1.0 - update.rear_brake_bias)); // Could be replaced with calculating value container
    handle.update_property(P_TELEMETRY_REAR_BRAKE_BIAS, Property::from(update.rear_brake_bias)); 
    handle.update_property(P_TELEMETRY_ENGINE_TURBO_BOOST_PRESSURE, Property::from(update.turbo_boost_pressure));
    handle.update_property(P_TELEMETRY_PHYSICAL_WHEEL_RANGE, Property::from(update.physical_steering_wheel_range)); // infrequently

    // handle.log_info(format!("Time: {}", handle.get_property_value(P_TELEMETRY_SESSION_ELAPSED_TIME).unwrap().to_duration().unwrap().0.as_secs_f64()));
}

fn read_scoring(_handle: &PluginHandle, update: PageScoring, state: &mut ReaderState) {
    let num_vehicles = if update.scoring_info.num_vehicles >= 0 && (update.scoring_info.num_vehicles as usize) <= MAX_MAPPED_VEHICLES {
        update.scoring_info.num_vehicles as usize
    } else {
        MAX_MAPPED_VEHICLES
    };

    for i in 0..num_vehicles {
        let veh = update.vehicles[i];

        if veh.is_player != 0 {
            state.player_vehicle_id = veh.id;
        }

        
        
    }

}


#[inline]
fn help_read_string(handle: &PluginHandle, slice: &[u8], cache: &mut String, property: PropertyHandle) {
    let read = String::from_utf8_lossy(slice);

    if read != cache.as_str() {
        // Change detected
        *cache = read.to_string();

        let fix = cache.to_string();
        handle.update_property(property, Property::Str(fix));
    }
}
