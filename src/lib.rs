use gst::glib;
use gstreamer as gst;

gst::gst_plugin_define!(
    gllcms,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
    // "MIT/X11",
    "unknown",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY"),
    env!("BUILD_REL_DATE")
);

mod gllcms;

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gllcms::register(plugin)
}
