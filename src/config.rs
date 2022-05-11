use crate::renderer::Renderer;

pub struct Config {
    pub renderer: Renderer,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            renderer: Renderer::stdout(),
        }
    }
}

impl Config {
    pub fn buffered() -> Self {
        Self {
            renderer: Renderer::buffered(),
        }
    }
}
