//! A fast, asynchronous terminal paging library for Rust. `minus` provides high
//! level functionalities to easily write a pager for any terminal application.
//! Due to the asynchronous nature of `minus`, the pager's data can be
//! **updated** (this needs the correct feature to be enabled).
//!
//! `minus` supports both [`tokio`] as well as [`async-std`] runtimes. What's
//! more, if you only want to use `minus` for serving static output, you can
//! simply opt out of these dynamic features, see the
//! [**Features**](crate#features) section below.
//!
//! ## Why this crate ?
//!
//! `minus` was started by me for my work on [`pijul`]. I was unsatisfied with
//! the existing options like [`pager`] and [`moins`].
//!
//! * [`pager`]:
//!     * Only provides functions to join the standard output of the current
//!       program to the standard input of external pager like `more` or `less`.
//!     * Due to this, to work within Windows, the external pagers need to be
//!       packaged along with the executable.
//!
//! * [`moins`]:
//!     * The output could only be defined once and for all. It is not asynchronous
//!       and does not support updating.
//!
//! The main goals of `minus` are to be very compact and as configurable as possible.
//! * `minus` provides a lot of configurablity to the end-application and this
//! configuration can be defined not just in compile-time but also in **runtime.** Your
//! entire configuration like the output displayed, prompt and line numbers are inside
//! a `Arc<Mutex>`, which means at any time you can lock the configuration, change
//! something, and voila minus will automatically update the screen
//!
//! * When using `minus`, you select what features you need and **nothing else**. See
//! [Features](crate#features) below
//!
//! [`tokio`]: https://crates.io/crates/tokio
//! [`async-std`]: https://crates.io/crates/async-std
//! [`pager`]: https://crates.io/crates/pager
//! [`moins`]: https://crates.io/crates/moins
//! [`pijul`]: https://pijul.org/
//!
//! ## Features
//!
//! * `async_std_lib`: Use this if you use [`async_std`] runtime in your
//! application
//! * `tokio_lib`:Use this if you are using [`tokio`] runtime for your application
//! * `static_output`: Use this if you only want to use `minus` for displaying static
//! output
//! * `search`: If you want searching capablities inside the feature

// When no feature is active this crate is unusable but contains lots of
// unused imports and dead code. To avoid useless warnings about this they
// are allowed when no feature is active.
#![cfg_attr(
    not(any(
        feature = "tokio_lib",
        feature = "async_std_lib",
        feature = "static_output"
    )),
    allow(unused_imports),
    allow(dead_code)
)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

mod error;
#[cfg(any(feature = "tokio_lib", feature = "async_std_lib"))]
mod rt_wrappers;
#[cfg(feature = "search")]
mod search;
#[cfg(feature = "static_output")]
mod static_pager;
mod utils;

#[cfg(any(feature = "tokio_lib", feature = "async_std_lib"))]
pub use rt_wrappers::*;
#[cfg(feature = "static_output")]
pub use static_pager::page_all;

#[cfg(any(feature = "tokio_lib", feature = "async_std_lib"))]
use async_mutex::Mutex;
pub use error::*;

#[cfg(any(feature = "tokio_lib", feature = "async_std_lib"))]
use std::sync::Arc;
pub use utils::LineNumbers;
#[cfg(feature = "search")]
use utils::SearchMode;
mod init;
// mod line;

// use line::{Line, WrappedLines};

#[cfg(any(feature = "tokio_lib", feature = "async_std_lib"))]
pub type PagerMutex = Arc<Mutex<Pager>>;

/// A struct containing basic configurations for the pager. This is used by
/// all initializing functions
///
/// ## Example
/// You can use any async runtime, but we are taking the example of [`tokio`]
///```rust,no_run
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     use minus::{Pager, LineNumbers, tokio_updating};
///     let mut pager = Pager::new();
///     pager.set_line_numbers(LineNumbers::AlwaysOn);
///     pager.set_prompt("A complex configuration");
///
///     // Normally, you would use `futures::join` to join the pager and the text
///     // updating function. We are doing this here to make the example simple
///     tokio_updating(pager.finish()).await?;
///     Ok(())
/// }
///```
///
/// For static output
///```rust,no_run
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///      let mut pager = minus::Pager::new();
///      pager.set_text("Hello");
///      pager.set_prompt("Example");
///      minus::page_all(pager)?;
///      Ok(())
/// }
///```
///
#[derive(Clone)]
pub struct Pager {
    /// The output that is displayed
    lines: Vec<Vec<String>>,
    /// Configuration for line numbers. See [`LineNumbers`]
    pub(crate) line_numbers: LineNumbers,
    /// The prompt displayed at the bottom
    prompt: String,
    /// Is the pager running
    running: bool,
    // Text which may have come that may be unwraped
    unwraped_text: String,
    /// The behaviour to do when user quits the program using `q` or `Ctrl+C`
    /// See [`ExitStrategy`] for available options
    exit_strategy: ExitStrategy,
    /// The upper mark of scrolling. It is kept private to prevent end-applications
    /// from mutating this
    pub(crate) upper_mark: usize,
    /// Stores the most recent search term
    #[cfg(feature = "search")]
    search_term: String,
    /// A temporary space to store modifications to the lines string
    #[cfg(feature = "search")]
    search_lines: WrappedLines,
    // Direction of search
    #[cfg(feature = "search")]
    search_mode: SearchMode,
    /// Lines where searches have a match
    #[cfg(feature = "search")]
    pub(crate) search_idx: Vec<u16>,
    /// Rows of the terminal
    pub(crate) rows: usize,
    /// Columns of the terminal
    pub(crate) cols: usize,
}

impl Pager {
    /// Initialize a new pager configuration
    ///
    /// Example
    /// ```
    /// let pager = minus::Pager::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Pager {
            lines: Vec::new(),
            line_numbers: LineNumbers::Disabled,
            upper_mark: 0,
            prompt: "minus".to_string(),
            exit_strategy: ExitStrategy::ProcessQuit,
            running: false,
            unwraped_text: String::new(),
            #[cfg(feature = "search")]
            search_term: String::new(),
            #[cfg(feature = "search")]
            search_lines: WrappedLines::new(),
            #[cfg(feature = "search")]
            search_mode: SearchMode::Unknown,
            #[cfg(feature = "search")]
            search_idx: Vec::new(),
            // Just to be safe in tests, keep at 1x1 size
            cols: 1,
            rows: 1,
        }
    }

    /// Set the output text to this `t`
    ///
    /// Note that unlike [`Pager::push_str`], this replaces the original text.
    /// If you want to append text, use the [`Pager::push_str`] function
    ///
    /// Example
    /// ```
    /// let mut pager = minus::Pager::new();
    /// pager.set_text("This is a line");
    /// ```
    pub fn set_text(&mut self, text: impl Into<String>) {
        if self.running {
            let text: String = text.into();
            // self.lines = WrappedLines::from(Line::from_str(&text.into(), self.cols));
            self.lines = text
                .lines()
                .map(|l| {
                    textwrap::wrap(l, self.cols)
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<String>>()
                })
                .collect();
        } else {
            self.unwraped_text = text.into();
        }
    }

    /// Set line number to this setting
    ///
    /// Example
    /// ```
    /// use minus::{Pager, LineNumbers};
    ///
    /// let pager = Pager::new().set_line_numbers(LineNumbers::Enabled);
    /// ```
    pub fn set_line_numbers(&mut self, l: LineNumbers) {
        self.line_numbers = l;
    }

    /// Set the prompt displayed at the prompt to `t`
    ///
    /// Example
    /// ```
    /// use minus::Pager;
    ///
    /// let mut pager = Pager::new();
    /// pager.set_prompt("my awesome program");
    /// ```
    pub fn set_prompt(&mut self, t: impl Into<String>) {
        self.prompt = t.into();
    }

    /// Return a [`PagerMutex`] from this [`Pager`]. This is gated on `tokio_lib` or
    /// `async_std_lib` feature
    ///
    /// Example
    /// ```
    /// use minus::Pager;
    ///
    /// let mut pager = Pager::new();
    /// pager.set_text("This output is paged");
    /// let _pager_mutex = pager.finish();
    /// ```
    #[must_use]
    #[cfg(any(feature = "tokio_lib", feature = "async_std_lib"))]
    pub fn finish(self) -> PagerMutex {
        Arc::new(Mutex::new(self))
    }

    /// Set the default exit strategy.
    ///
    /// This controls how the pager will behave when the user presses `q` or `Ctrl+C`.
    /// See [`ExitStrategy`] for available options
    ///
    /// ```
    /// use minus::{Pager, ExitStrategy};
    ///
    /// let mut pager = Pager::new();
    /// pager.set_exit_strategy(ExitStrategy::ProcessQuit);
    /// ```
    pub fn set_exit_strategy(&mut self, strategy: ExitStrategy) {
        self.exit_strategy = strategy;
    }

    /// Returns the appropriate text for displaying.
    ///
    /// Nrmally it will return `self.lines`
    /// In case of a search, `self.search_lines` is returned
    pub(crate) fn get_lines(&self) -> Vec<Vec<String>> {
        #[cfg(feature = "search")]
        if self.search_term.is_empty() {
            self.lines.clone()
        } else {
            self.search_lines.clone()
        }
        #[cfg(not(feature = "search"))]
        self.lines.clone()
    }

    /// Appends text to the pager output
    ///
    /// This function will automatically split the lines, if they overflow
    /// the number of terminal columns
    ///
    /// ```
    /// let mut pager = minus::Pager::new();
    /// pager.push_str("This is some text");
    /// ```
    pub fn push_str(&mut self, text: impl Into<String>) {
        let text: String = text.into();
        if self.running {
            text.lines().for_each(|l| {
                self.lines.push(
                    textwrap::wrap(l, self.cols)
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<String>>(),
                )
            });
        } else {
            self.unwraped_text.push_str(&text);
        }
    }
    /// Prepare the terminal
    ///
    /// Sets the rows and columns of the terminal inside the pager.
    /// Also prepares any unwraped text that might have come before running
    ///
    /// # Panics
    /// This function panics if te pager is already running  
    pub(crate) fn prepare(&mut self) -> Result<(), error::AlternateScreenPagingError> {
        let (cols, rows) = crossterm::terminal::size().map_err(|e| {
            error::AlternateScreenPagingError::HandleEvent(error::TermError::from(e))
        })?;
        self.cols = cols.into();
        self.rows = rows.into();
        if self.running {
            panic!("prepare() called after the pager is started to run")
        } else {
            self.running = true;
            self.lines = self
                .unwraped_text
                .lines()
                .map(|l| {
                    textwrap::wrap(l, self.cols)
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<String>>()
                })
                .collect();
        }
        Ok(())
    }
    /// Readjust the text to new terminal size
    pub(crate) fn readjust_wraps(&mut self) {
        for line in self.lines.iter_mut() {
            *line = textwrap::wrap(&line.join(""), self.cols)
                .iter()
                .map(|c| c.to_string())
                .collect()
        }
    }
}

impl std::default::Default for Pager {
    fn default() -> Self {
        Pager::new()
    }
}

/// Behaviour that happens when the pager is exitted
#[derive(PartialEq, Clone)]
pub enum ExitStrategy {
    /// Kill the entire application immediately.
    ///
    /// This is the prefered option if paging is the last thing you do. For example,
    /// the last thing you do in your program is reading from a file or a database and
    /// paging it concurrently
    ///
    /// **This is the default strategy.**
    ProcessQuit,
    /// Kill the pager only.
    ///
    /// This is the prefered option if you want to do more stuff after exiting the pager. For example,
    /// if you've file system locks or you want to close database connectiions after
    /// the pager has done i's job, you probably want to go for this option
    PagerQuit,
}
