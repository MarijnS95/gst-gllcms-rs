use gst::glib;
use gst::subclass::prelude::*;

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

glib::glib_wrapper! {
    pub struct GlLcms(ObjectSubclass<gllcms::GlLcms>) @extends gst_gl::GLFilter, gst_gl::GLBaseFilter;
}

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        gllcms::GlLcms::NAME,
        gst::Rank::None,
        gllcms::GlLcms::get_type(),
    )
}
