use emocis_cclg::{LanguageAdapter, TypeScriptAdapter};

#[test]
fn test_vue_sfc_byte_preservation() {
    let ts_adapter = TypeScriptAdapter::new();
    let vue_content = r#"<template>
  <div class="header">
    <h1>Kasir Citra 😀 (isOnline ? true : false)</h1>
    <button @click="handleDeviceUnlock()">Unlock</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const isLocked = ref<boolean>(true);

async function handleDeviceUnlock() {
    isLocked.value = false;
}

async function checkPin(pin: string) {
    if (pin === "1234") {
        await handleDeviceUnlock();
    }
}
</script>

<style scoped>
.header { color: red; }
</style>
"#;

    let graph = ts_adapter.parse_file("proj", "mobile/src/App.vue", vue_content).unwrap();
    println!("VUE NODES:");
    for n in &graph.nodes {
        println!("  node: '{}' ({:?}) at {:?}", n.name, n.kind, n.source);
    }

    let fn_names: Vec<_> = graph.nodes.iter().filter(|n| n.kind == emocis_core::NodeKind::Function).map(|n| n.name.as_str()).collect();
    assert!(fn_names.contains(&"handleDeviceUnlock"), "Must extract handleDeviceUnlock cleanly");
    assert!(fn_names.contains(&"checkPin"), "Must extract checkPin cleanly");
    for name in &fn_names {
        assert!(!name.contains('\n'), "Function name should never contain newline: {:?}", name);
        assert!(!name.contains('('), "Function name should never contain paren: {:?}", name);
        assert!(!name.contains(';'), "Function name should never contain semicolon: {:?}", name);
    }
}

#[test]
fn test_generic_call_linking() {
    let ts_adapter = TypeScriptAdapter::new();
    let code = r#"
async function idbGet<T>(key: string): Promise<T> {
    return null as any;
}

export async function testCaller() {
    const val = await idbGet<CryptoKey>("my-key");
    const val2 = idbGet<CryptoKey>("another-key");
    return val;
}
"#;
    let graph = ts_adapter.parse_file("proj", "test.ts", code).unwrap();
    let call_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.kind == emocis_core::EdgeKind::Calls && e.from.contains("testCaller"))
        .collect();

    assert!(!call_edges.is_empty(), "Must have call edges from testCaller");
    for edge in call_edges {
        assert!(
            edge.to.ends_with(":idbGet:function"),
            "Target of call edge must be idbGet function, got {}",
            edge.to
        );
    }
}
