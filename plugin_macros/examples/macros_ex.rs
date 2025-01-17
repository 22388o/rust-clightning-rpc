//! plugin macros usage example.
extern crate clightningrpc_plugin_macros;
use clightningrpc_plugin_macros::{add_plugin_rpc, rpc_method};
use serde_json::{json, Value};

use clightningrpc_plugin::add_rpc;
use clightningrpc_plugin::commands::RPCCommand;
use clightningrpc_plugin::plugin::Plugin;

#[rpc_method(
    rpc_name = "foo",
    description = "This is a simple and short description"
)]
pub fn foo_rpc(_plugin: Plugin<()>, _request: Value) -> Value {
    /// The name of the parameters can be used only if used, otherwise can be omitted
    /// the only rules that the macros require is to have a propriety with the following rules:
    /// - Plugin as _plugin
    /// - CLN JSON request as _request
    /// The function parameter can be specified in any order.
    json!({"is_dynamic": _plugin.dynamic, "rpc_request": _request})
}

fn main() {
    // as fist step you need to make a new plugin instance
    // more docs about Plugin struct is provided under the clightning_plugin crate
    let mut plugin = Plugin::new((), true);

    // The macros helper that help to register a RPC method with the name
    // without worry about all the rules of the library
    add_plugin_rpc!(plugin, "foo");

    plugin.start();
}
