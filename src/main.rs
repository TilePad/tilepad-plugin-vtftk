use plugin::VtftkPlugin;
use tilepad_plugin_sdk::{setup_tracing, start_plugin};
use tokio::task::LocalSet;

pub mod plugin;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    setup_tracing();

    let local_set = LocalSet::new();
    let plugin = VtftkPlugin::new();

    local_set.run_until(start_plugin(plugin)).await;
}
