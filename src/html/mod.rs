//! MDBook's HTML renderer.

#![allow(missing_docs)]

mod config;

pub use self::config::{HtmlConfig, Playpen};

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use handlebars::Handlebars;
use serde_json::value::{Map, Value};

use book::{Book, BookItem, Chapter};
use theme::Theme;
use config::Config;
use renderer::{RenderContext, Renderer};
use renderer::html_handlebars::helpers::{next, previous, RenderToc};
use errors::*;
use utils;

type JsonObject = Map<String, Value>;

pub struct HtmlRenderer;

impl HtmlRenderer {
    /// Render all the chapters and write them to the output directory.
    fn render_chapters(
        &self,
        book: &Book,
        global_ctx: &JsonObject,
        dest: &Path,
        title: Option<&String>,
    ) -> Result<()> {
        for item in book.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                let content = self.render_chapter(ch, global_ctx, title)
                    .chain_err(|| format!("Unable to render \"{}\"", ch.name))?;

                let output_file = dest.join(&ch.path);
                write_all(&output_file, &content).chain_err(|| {
                    format!(
                        "Writing chapter content to \"{}\" failed",
                        output_file.display()
                    )
                })?;
            }
        }

        Ok(())
    }

    /// Render a single chapter as HTML.
    fn render_chapter(
        &self,
        ch: &Chapter,
        global_ctx: &JsonObject,
        book_title: Option<&String>,
    ) -> Result<String> {
        let title = match book_title {
            Some(book_title) => format!("{} - {}", book_title, ch.name),
            None => ch.name.clone(),
        };

        let mut chapter_ctx = json!({
            "path": ch.path,
            "content": ch.content,
            "chapter_title": ch.name,
            "title": title,
            "path_to_root": utils::fs::path_to_root(&ch.path),
        });

        // update the render context with our book's global information
        match chapter_ctx {
            Value::Object(ref mut obj) => obj.extend(global_ctx.clone()),
            _ => unreachable!(),
        }

        // TODO: Pass this through pulldown-cmark and transform stuff appropriately
        unimplemented!()
    }
}

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

        let global_ctx = construct_global_context(&ctx.config, &cfg, &ctx.book);

        let theme_dir = cfg.theme_dir(&ctx.root);
        let static_assets = Theme::new(theme_dir);
        let hbs = load_handlebars_engine(&static_assets, &cfg)?;

        self.render_chapters(
            &ctx.book,
            &global_ctx,
            &ctx.destination,
            ctx.config.book.title.as_ref(),
        )?;

        unimplemented!()
    }
}

fn load_handlebars_engine(assets: &Theme, cfg: &HtmlConfig) -> Result<Handlebars> {
    debug!("Loading the template engine");
    let mut hbs = Handlebars::new();

    hbs.register_template_string("index", String::from_utf8(assets.index.clone())?)?;
    hbs.register_partial("header", String::from_utf8(assets.header.clone())?)?;

    hbs.register_helper(
        "toc",
        Box::new(RenderToc {
            no_section_label: cfg.no_section_label,
        }),
    );
    hbs.register_helper("previous", Box::new(previous));
    hbs.register_helper("next", Box::new(next));

    Ok(hbs)
}

fn construct_global_context(cfg: &Config, html_config: &HtmlConfig, book: &Book) -> JsonObject {
    let title = cfg.book.title.as_ref();
    let description = cfg.book.description.as_ref();
    let authors = cfg.book.authors.as_slice();
    let ga = html_config.google_analytics.as_ref();
    let mathjax_enabled = html_config.mathjax_support;
    let livereload = html_config.livereload_url.as_ref();

    let toc_info = create_toc_info(book);

    let mut context = json!({
        "language": "en",
        "book_title": title,
        "description": description,
        "livereload": livereload,
        "authors": authors,
        "google_analytics": ga,
        "favicon": "favicon.ico",
        "mathjax_support": mathjax_enabled,
        "chapters": toc_info,
        "playpens_editable": html_config.playpen.editable,
    });

    if html_config.playpen.editable {
        let extra_info = json!({
            "editor_js": "editor.js",
            "ace_js": "ace.js",
            "mode_rust_js": "mode-rust.js",
            "theme_dawn_js": "theme-dawn.js",
            "theme_tomorrow_night_js": "theme-tomorrow_night.js",
        });

        let context = context.as_object_mut().expect("unreachable");
        match extra_info {
            Value::Object(extra) => context.extend(extra),
            _ => unreachable!(),
        }
    }

    match context {
        Value::Object(obj) => obj,
        _ => unreachable!(),
    }
}

/// Inspects the book and creates a simplified schematic of its contents. Mainly
/// for use in the TOC, but 3rd parties can use it too.
fn create_toc_info(book: &Book) -> Value {
    let mut chapters = Vec::new();

    for item in book.iter() {
        let obj = match *item {
            BookItem::Chapter(ref ch) => {
                let section = ch.number.as_ref().map(|s| s.to_string());
                let mut chapter = json!({
                    "name": ch.name,
                    "section": section,
                    "path": ch.path,
                });

                chapter
            }
            BookItem::Separator => json!({
                "spacer": "_spacer_",
            }),
        };

        chapters.push(obj);
    }

    Value::Array(chapters)
}

fn write_all<P: AsRef<Path>, D: AsRef<[u8]>>(location: P, data: D) -> Result<()> {
    let location = location.as_ref();
    let data = data.as_ref();

    if let Some(parent) = location.parent() {
        fs::create_dir_all(parent).chain_err(|| "Unable to create parent directories")?;
    }

    File::create(location)
        .chain_err(|| "Couldn't open the file for writing")?
        .write_all(data)
        .chain_err(|| "Error encountered writing data")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use book::SectionNumber;

    #[test]
    fn load_the_handlebars_template_using_defaults() {
        let cfg = HtmlConfig::default();
        let theme = Theme::new("");
        let got = load_handlebars_engine(&theme, &cfg).unwrap();

        let templates = got.get_templates();

        assert!(templates.contains_key("index"));
        assert!(templates.contains_key("header"));
    }

    #[test]
    fn create_the_toc_summary() {
        let mut book = Book::new();
        let mut first = Chapter::new("First", "# First Chapter".to_string(), "first.md");
        first.number = Some(SectionNumber(vec![1]));
        let mut nested = Chapter::new("Nested", "# Nested Chapter".to_string(), "first/nested.md");
        nested.number = Some(SectionNumber(vec![1, 1]));
        first.sub_items.push(BookItem::Chapter(nested));
        book.push_item(first);
        book.push_item(BookItem::Separator);
        book.push_item(Chapter::new(
            "Second",
            "# Second Chapter".to_string(),
            "second.md",
        ));

        let should_be = json!([
            {
                "section": "1.",
                "name": "First",
                "path": "first.md",
            },
            {
                "section": "1.1.",
                "name": "Nested",
                "path": "first/nested.md",
            },
            {"spacer": "_spacer_"},
            {
                "name": "Second",
                "path": "second.md",
                "section": null,
            },
        ]);

        let got = create_toc_info(&book);
        assert_eq!(got, should_be);
    }
}
