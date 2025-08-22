//! Plugin API

pub trait PluginApi {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self) -> anyhow::Result<()>;
    fn shutdown(&mut self) -> anyhow::Result<()>;
}
