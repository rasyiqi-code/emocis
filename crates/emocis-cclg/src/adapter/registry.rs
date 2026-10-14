use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use super::{LanguageAdapter, PythonAdapter, RustAdapter, TypeScriptAdapter};

pub struct AdapterRegistry {
    adapters: Vec<Arc<dyn LanguageAdapter>>,
    extension_map: HashMap<String, Arc<dyn LanguageAdapter>>,
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            adapters: Vec::new(),
            extension_map: HashMap::new(),
        };

        // Register default built-in adapters
        registry.register(Arc::new(TypeScriptAdapter::new()));
        registry.register(Arc::new(RustAdapter::new()));
        registry.register(Arc::new(PythonAdapter::new()));

        registry
    }

    pub fn register(&mut self, adapter: Arc<dyn LanguageAdapter>) {
        for ext in adapter.supported_extensions() {
            self.extension_map.insert(ext.to_string(), Arc::clone(&adapter));
        }
        self.adapters.push(adapter);
    }

    pub fn get_adapter_for_path(&self, file_path: &Path) -> Option<Arc<dyn LanguageAdapter>> {
        let ext = file_path.extension().and_then(|e| e.to_str())?;
        self.extension_map.get(ext).cloned()
    }

    pub fn is_supported(&self, file_path: &Path) -> bool {
        self.get_adapter_for_path(file_path).is_some()
    }
}
