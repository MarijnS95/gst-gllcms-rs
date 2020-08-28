use gst::glib;
use gst::prelude::*;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::*;
use gst_gl::*;
use gstreamer as gst;
use gstreamer_base as gst_base;
use gstreamer_gl as gst_gl;
use gstreamer_video as gst_video;

use glib::subclass;
use glib::subclass::prelude::*;

use once_cell::sync::Lazy;
use std::sync::Mutex;

struct State {
    in_info: gst_video::VideoInfo,
    out_info: gst_video::VideoInfo,
}

struct GlLcms {
    state: Mutex<Option<State>>,
}

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
    type ParentType = GLFilter;
    type Instance = gst::subclass::ElementInstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    // This macro provides some boilerplate
    glib::glib_object_subclass!();

    fn new() -> Self {
        Self {
            state: Mutex::new(None),
        }
    }

    fn class_init(klass: &mut Self::Class) {
        klass.set_metadata(
            "Rust LCMS2-based color correction in OpenGL",
            "Filter/Effect/Converter/Video",
            env!("CARGO_PKG_DESCRIPTION"),
            env!("CARGO_PKG_AUTHORS"),
        );

        // klass.configure(
        //     gst_base::subclass::BaseTransformMode::NeverInPlace,
        //     false,
        //     false,
        // );

        GLFilter::add_rgba_pad_templates(klass)
    }
}

unsafe impl IsSubclassable<GlLcms> for GLFilterClass {
    fn override_vfuncs(&mut self) {
        <glib::ObjectClass as IsSubclassable<GlLcms>>::override_vfuncs(self);
        unsafe {
            let klass = &mut *(self as *mut Self as *mut GLFilterClass);
        }
    }
}

impl ObjectImpl for GlLcms {
    glib::glib_object_impl!();
}
impl ElementImpl for GlLcms {}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        GlLcms::NAME,
        gst::Rank::None,
        GlLcms::get_type(),
    )
}
