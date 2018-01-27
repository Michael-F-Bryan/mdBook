use std::path::PathBuf;

/// Configuration for tweaking how the the HTML renderer handles the playpen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Playpen {
    /// The path to the editor to use. Defaults to the [Ace Editor].
    ///
    /// [Ace Editor]: https://ace.c9.io/
    pub editor: PathBuf,
    /// Should playpen snippets be editable? Defaults to `false`.
    pub editable: bool,
}

impl Default for Playpen {
    fn default() -> Playpen {
        Playpen {
            editor: PathBuf::from("ace"),
            editable: false,
        }
    }
}

/// Configuration for the HTML renderer.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct HtmlConfig {
    /// The theme directory, if specified.
    pub theme: Option<PathBuf>,
    /// Use "smart quotes" instead of the usual `"` character.
    pub curly_quotes: bool,
    /// Should mathjax be enabled?
    pub mathjax_support: bool,
    /// An optional google analytics code.
    pub google_analytics: Option<String>,
    /// Additional CSS stylesheets to include in the rendered page's `<head>`.
    pub additional_css: Vec<PathBuf>,
    /// Additional JS scripts to include at the bottom of the rendered page's
    /// `<body>`.
    pub additional_js: Vec<PathBuf>,
    /// Playpen settings.
    pub playpen: Playpen,
    /// This is used as a bit of a workaround for the `mdbook serve` command.
    /// Basically, because you set the websocket port from the command line, the
    /// `mdbook serve` command needs a way to let the HTML renderer know where
    /// to point livereloading at, if it has been enabled.
    ///
    /// This config item *should not be edited* by the end user.
    #[doc(hidden)]
    pub livereload_url: Option<String>,
    /// Should section labels be rendered?
    pub no_section_label: bool,
}
