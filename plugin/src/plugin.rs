//! Core of the plugin API
//!
//! Unofficial API interface to develop plugin in Rust.
use crate::commands::json_utils::{add_str, init_success_response};
use crate::commands::{
    builtin::{InitRPC, ManifestRPC},
    types::{RPCHookInfo, RPCMethodInfo},
    RPCCommand,
};
use crate::types::{LogLevel, RpcOption};
use clightningrpc_common::types::Request;
use std::collections::{HashMap, HashSet};
use std::string::String;
use std::{io, io::Write};

#[derive(Clone)]
#[allow(dead_code)]
pub struct Plugin<T>
where
    T: Clone,
{
    state: T,
    /// all the option contained inside the
    /// hash map.
    pub option: HashSet<RpcOption>,
    /// all the options rpc method that the
    /// plugin need to support, included the builtin rpc method.
    pub rpc_method: HashMap<String, Box<dyn RPCCommand<T>>>,
    /// keep the info of the method in a separate list
    /// FIXME: move the RPCMethodInfo as key of the rpc_method map.
    pub rpc_info: HashSet<RPCMethodInfo>,
    /// all the hook where the plugin is register during the configuration
    pub rpc_hook: HashMap<String, Box<dyn RPCCommand<T>>>,
    /// keep all the info about the hooks in a separate set.
    /// FIXME: put the RPCHookInfo as key of the hash map.
    pub hook_info: HashSet<RPCHookInfo>,
    /// all the notification that the plugin is register on
    pub rpc_nofitication: HashMap<String, Box<dyn RPCCommand<T>>>,
    /// mark a plugin as dynamic, in this way the plugin can be run
    /// from core lightning without stop the lightningd deamon
    pub dynamic: bool,
}

impl<'a, T: 'a + Clone> Plugin<T> {
    pub fn new(state: T, dynamic: bool) -> Self {
        return Plugin {
            state,
            option: HashSet::new(),
            rpc_method: HashMap::new(),
            rpc_info: HashSet::new(),
            rpc_hook: HashMap::new(),
            hook_info: HashSet::new(),
            rpc_nofitication: HashMap::new(),
            dynamic,
        };
    }

    pub fn log(&self, level: LogLevel, msg: &str) -> &Self {
        let mut writer = io::stdout();
        let mut log_req = init_success_response(40);
        add_str(&mut log_req, "level", &level.to_string()[0..]);
        add_str(&mut log_req, "message", msg);
        writer
            .write_all(serde_json::to_string(&log_req).unwrap().as_bytes())
            .unwrap();
        writer.flush().unwrap();
        self
    }

    pub fn add_opt(
        &mut self,
        name: &str,
        opt_type: &str,
        def_val: Option<String>,
        description: &str,
        deprecated: bool,
    ) -> &mut Self {
        self.option.insert(RpcOption {
            name: name.to_string(),
            opt_typ: opt_type.to_string(),
            default: def_val,
            description: description.to_string(),
            deprecated,
        });
        self
    }

    // FIXME: adding the long description as parameter
    pub fn add_rpc_method<F: 'static>(
        &'a mut self,
        name: &str,
        usage: &str,
        description: &str,
        callback: F,
    ) -> &mut Self
    where
        F: RPCCommand<T> + 'static,
    {
        self.rpc_method.insert(name.to_owned(), Box::new(callback));
        self.rpc_info.insert(RPCMethodInfo {
            name: name.to_string(),
            usage: usage.to_string(),
            description: description.to_string(),
            long_description: description.to_string(),
            deprecated: false,
        });
        self
    }

    fn call_rpc_method(&'a mut self, name: &str, params: &serde_json::Value) -> serde_json::Value {
        let command = self.rpc_method.get(name).unwrap().clone();
        command.call(self, params)
    }

    pub fn register_hook<F: 'static>(
        &'a mut self,
        hook_name: &str,
        before: Option<Vec<String>>,
        after: Option<Vec<String>>,
        callback: F,
    ) -> &mut Self
    where
        F: RPCCommand<T> + 'static,
    {
        self.rpc_hook
            .insert(hook_name.to_owned(), Box::new(callback));
        self.hook_info.insert(RPCHookInfo {
            name: hook_name.to_owned(),
            before,
            after,
        });
        self
    }

    pub fn register_notification<F: 'static>(&mut self, name: &str, callback: F) -> &mut Self
    where
        F: 'static + RPCCommand<T> + Clone,
    {
        self.rpc_nofitication
            .insert(name.to_owned(), Box::new(callback));
        self
    }

    pub fn start(&'a mut self) {
        let reader = io::stdin();
        let mut writer = io::stdout();
        let mut buffer = String::new();

        self.rpc_method
            .insert("getmanifest".to_owned(), Box::new(ManifestRPC {}));
        self.rpc_method
            .insert("init".to_owned(), Box::new(InitRPC {}));
        // FIXME: core lightning end with the double endline, so this can cause
        // problem for some input reader.
        // we need to parse the writer, and avoid this while loop
        loop {
            let _ = reader.read_line(&mut buffer);
            let req_str = buffer.to_string();
            if req_str.trim().is_empty() {
                continue;
            }
            buffer.clear();
            let request: Request<serde_json::Value> = serde_json::from_str(&req_str).unwrap();
            let response = self.call_rpc_method(request.method, &request.params);
            let mut rpc_response = init_success_response(request.id);
            rpc_response["result"] = response;
            writer
                .write_all(serde_json::to_string(&rpc_response).unwrap().as_bytes())
                .unwrap();
            writer.flush().unwrap();
        }
    }
}
