use std::fmt;

use nu_ansi_term::{Color, Style};
use tracing::{log::Level, Event, Subscriber};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::time::{SystemTime, UtcTime};
use tracing_subscriber::fmt::{
    format::{self, FormatEvent, FormatFields},
    FmtContext, FormattedFields,
};
use tracing_subscriber::registry::{LookupSpan, self};
use tracing_subscriber::{
    fmt::format::{FmtSpan, Writer},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub fn init_tracing_subscriber() {
    // setup tracing subscriber first and foremost
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "marshalrwr=debug,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .event_format(ConsoleFormatter::default()),
        )
        .init();
}

pub struct ConsoleFormatter {
    pub display_timestamp: bool,
    pub display_target: bool,
    pub display_level: bool,
    pub display_thread_id: bool,
    pub display_thread_name: bool,
    pub display_filename: bool,
    pub display_line_number: bool,
    pub display_event_fields: bool,
}

impl Default for ConsoleFormatter {
    fn default() -> Self {
        ConsoleFormatter {
            display_timestamp: true,
            display_target: true,
            display_level: true,
            display_thread_id: false,
            display_thread_name: false,
            display_filename: false,
            display_line_number: false,
            display_event_fields: true,
        }
    }
}

impl ConsoleFormatter {
    #[inline]
    fn format_timestamp(&self, writer: &mut Writer<'_>) -> fmt::Result {
        if !self.display_timestamp {
            return Ok(());
        }
        // let t = UtcTime::rfc_3339();
        let t = SystemTime;
        let time_style = Style::new().dimmed();
        if writer.has_ansi_escapes() {
            write!(writer, "{}", time_style.prefix())?;
        }
        t.format_time(writer)?;
        if writer.has_ansi_escapes() {
            write!(writer, "{}", time_style.suffix())?;
        }
        // insert the space ready for next column
        writer.write_char(' ')
    }

    #[inline]
    fn format_level(&self, writer: &mut Writer<'_>, level: &tracing::Level) -> fmt::Result {
        if !self.display_level {
            return Ok(());
        }
        let emoji = match level {
            &tracing::Level::ERROR => "❌",
            &tracing::Level::WARN => "⚠",
            &tracing::Level::INFO => "ℹ",
            &tracing::Level::DEBUG => "🔎",
            &tracing::Level::TRACE => "⚙",
        };
        let emoji_width = unicode_width::UnicodeWidthStr::width_cjk(emoji);
        let num_spaces = 3 - emoji_width;
        write!(writer, "{}", emoji)?;
        // insert the space(s) ready for next column
        for _ in 0..num_spaces {
            writer.write_char(' ')?;
        }
        Ok(())
    }
}

impl<S, N> FormatEvent<S, N> for ConsoleFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();
        let lvl = metadata.level();

        // write timestamp
        self.format_timestamp(&mut writer)?;
        // write level
        self.format_level(&mut writer, lvl)?;
        // write target
        if self.display_target {
            write!(&mut writer, "{}:", metadata.target())?;
            if self.display_line_number {
                if let Some(ln) = metadata.line() {
                    write!(&mut writer, "{}:", ln)?;
                }
            }
            writer.write_char(' ')?;
        }

        // write the message?!
        ctx.format_fields(writer.by_ref(), event)?;
        writer.write_char(' ')?;
        
        // include event fields if not INFO level
        if lvl != &tracing::Level::INFO {
            if self.display_event_fields {
                let event_field_style = Style::new().dimmed().italic();
                if writer.has_ansi_escapes() {
                    write!(writer, "{}", event_field_style.prefix())?;
                }
                writer.write_char('{')?;
                // output the event fields, based on tracing_subscriber Compact FormatEvent impl
                for span in ctx.event_scope().into_iter().flat_map(registry::Scope::from_root) {
                    let exts = span.extensions();
                    if let Some(fields) = exts.get::<FormattedFields<N>>() {
                        if !fields.is_empty() {
                            write!(writer, "{}", &fields.fields)?;
                        }
                    }
                }
                writer.write_char('}')?;
                if writer.has_ansi_escapes() {
                    write!(writer, "{}", event_field_style.suffix())?;
                }
            }
        }

        // finish by ending the line
        writeln!(writer)
    }
}
