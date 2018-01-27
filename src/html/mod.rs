//! MDBook's HTML renderer.

#![allow(missing_docs)]

mod config;

pub use self::config::{HtmlConfig, Playpen};

use std::path::Path;
use handlebars::Handlebars;

use renderer::{RenderContext, Renderer};
use errors::*;

pub struct HtmlRenderer;

impl Renderer for HtmlRenderer {
    fn name(&self) -> &str {
        "HTML"
    }

    fn render(&self, ctx: &RenderContext) -> Result<()> {
        let cfg: HtmlConfig = ctx.config
            .get_deserialized("output.html")
            .unwrap_or_default();

        if log_enabled!(::log::Level::Debug) {
            for line in format!("{:#?}", cfg).lines() {
                debug!("{}", line);
            }
        }

        let hbs = load_handlebars_engine(&ctx.root, &cfg)?;

        unimplemented!()
    }
}

fn load_handlebars_engine(root: &Path, cfg: &HtmlConfig) -> Result<Handlebars> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_the_handlebars_template_using_defaults() {
        let cfg = HtmlConfig::default();
        let got = load_handlebars_engine(Path::new(""), &cfg).unwrap();

        let templates = got.get_templates();

        assert!(templates.contains_key("index"));
        assert!(templates.contains_key("header"));
    }
}
