use datarace_plugin_api::{macros::generate_property_handle, wrappers::{DataStoreReturnCode, PluginHandle, Property, PropertyHandle}};

const P_EXTRA: PropertyHandle = generate_property_handle!("rF2-Reader.extra");

/// Creates the property handles during init
pub(crate) fn init_properties(handle: &PluginHandle) -> Result<(), String> {
    create_prop(&handle, "extra", P_EXTRA, Property::None)?;

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

#[allow(dead_code)]
pub(crate) fn update_properties(_handle: &PluginHandle) -> Result<(), String> {

    Ok(())
}
