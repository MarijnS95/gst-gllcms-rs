use gst::glib;
use gst::prelude::*;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::*;
use gstreamer as gst;
use gstreamer_base as gst_base;
use gstreamer_video as gst_video;

use glib::subclass;
use glib::subclass::prelude::*;

// use std::i32;
use std::sync::Mutex;

use once_cell::sync::Lazy;

struct GlLcms {}

impl GlLcms {}

static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        "gllcms",
        gst::DebugColorFlags::empty(),
        Some("Rust LCMS2-based color correction in OpenGL"),
    )
});

impl ObjectSubclass for GlLcms {
    const NAME: &'static str = "gllcms";
    type ParentType = gst_base::BaseTransform;
    type Instance = gst::subclass::ElementInstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    // This macro provides some boilerplate
    glib::glib_object_subclass!();

    fn new() -> Self {
        Self {}
    }

    fn class_init(klass: &mut subclass::simple::ClassStruct<Self>) {
        klass.set_metadata(
            "Rust LCMS2-based color correction in OpenGL",
            "Filter/Effect/Converter/Video",
            env!("CARGO_PKG_DESCRIPTION"),
            env!("CARGO_PKG_AUTHORS"),
        );

        klass.configure(
            gst_base::subclass::BaseTransformMode::NeverInPlace,
            false,
            false,
        );
    }
}

impl ObjectImpl for GlLcms {}
impl ElementImpl for GlLcms {}
impl BaseTransformImpl for GlLcms {}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        GlLcms::NAME,
        gst::Rank::None,
        GlLcms::get_type(),
    )
}
