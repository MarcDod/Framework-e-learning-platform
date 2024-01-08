use actix_web::http::Method;
use serde::Deserialize;
use std::{collections::HashMap, fs};
use toml;

use crate::models::util::AccessType;

#[derive(Clone)]
pub struct PermissionConfig {
    path_permissions: HashMap<String, HashMap<Method, (String, Option<String>, Vec<AccessType>)>>,
}

#[derive(Debug, Deserialize)]
struct Config {
    config: ConfigEntry,
}

#[derive(Debug, Deserialize)]
struct ConfigEntry {
    ressources: Vec<PermissionEntry>,
}

#[derive(Debug, Deserialize)]
struct PermissionEntry {
    value: String,
    routes: Vec<PermissionRoute>,
}

#[derive(Debug, Deserialize)]
struct PermissionRoute {
    path: String,
    #[serde(default)]
    param: Option<String>,
    method: String,
    #[serde(default)]
    required_access_types: Vec<AccessType>,
}

fn get_method_from_string(method_string: String) -> Method {
    Method::from_bytes(method_string.as_bytes()).unwrap()
}

impl PermissionConfig {
    pub fn new() -> Self {
        let mut permission_config = Self {
            path_permissions: HashMap::new(),
        };

        let relative_path = "src/assets/Permission.toml";
        let mut toml_path = std::env::current_dir().unwrap();
        toml_path.push(relative_path);

        let toml_content = fs::read_to_string(toml_path).unwrap();

        let config: Config = toml::from_str(&toml_content).expect("Failed to parse TOML");

        for permission in config.config.ressources {
            for route in permission.routes {
                permission_config.add_permission(
                    &route.path,
                    route.param,
                    get_method_from_string(route.method),
                    permission.value.to_string(),
                    route.required_access_types,
                );
            }
        }

        permission_config
    }

    fn add_permission(
        &mut self,
        path: &str,
        group_pattern: Option<String>,
        method: Method,
        permission: String,
        required_addons: Vec<AccessType>,
    ) {
        let method_map = self
            .path_permissions
            .entry(path.to_string())
            .or_insert(HashMap::new());
        method_map.insert(
            method,
            (permission, group_pattern, required_addons),
        );
    }

    pub fn get_permission_and_group_pattern(
        &self,
        path: &str,
        method: Method,
    ) -> Option<&(String, Option<String>, Vec<AccessType>)> {
        self.path_permissions
            .get(path)
            .and_then(|method_map| method_map.get(&method))
    }
}
