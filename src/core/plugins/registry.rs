use crate::core::volkiwithstds::collections::Vec;
use crate::core::volkiwithstds::path::Path;
use crate::veprintln;

use super::protocol::{PluginRequest, PluginResponse};
use super::resolver;
use super::runner;
use super::types::{PluginError, PluginSpec, ResolvedPlugin};

pub struct PluginRegistry {
    plugins: Vec<(PluginSpec, ResolvedPlugin)>,
}

impl PluginRegistry {
    pub fn load(specs: &[PluginSpec], project_dir: &Path) -> Self {
        let mut plugins = Vec::new();
        for spec in specs {
            match resolver::resolve(spec, project_dir) {
                Ok(resolved) => {
                    plugins.push((spec.clone(), resolved));
                }
                Err(e) => {
                    veprintln!("warning: plugin '{}': {}", spec.name, e);
                }
            }
        }
        PluginRegistry { plugins }
    }

    pub fn invoke_hook(
        &self,
        request_builder: &dyn Fn(&PluginSpec) -> PluginRequest,
    ) -> Vec<Result<PluginResponse, PluginError>> {
        self.plugins
            .iter()
            .map(|(spec, resolved)| {
                let request = request_builder(spec);
                runner::invoke(resolved, &request)
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}
