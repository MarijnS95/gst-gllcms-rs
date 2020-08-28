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
    type ParentType = gst_base::BaseTransform;
    type Instance = gst::subclass::ElementInstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    // This macro provides some boilerplate
    glib::glib_object_subclass!();

    fn new() -> Self {
        Self {
            state: Mutex::new(None),
        }
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

        let caps = gst::Caps::builder("video/x-raw")
            .features(&[
                &CAPS_FEATURE_MEMORY_GL_MEMORY,
                // TODO: meta:GstVideoOverlayComposition? That'd be only for input though!
                // &gst_video::CAPS_FEATURE_META_GST_VIDEO_OVERLAY_COMPOSITION,
            ])
            .field("format", &gst_video::VideoFormat::Rgba.to_string())
            .field("texture-target", &gst::List::new(&[&"2D", &"external-oes"]))
            // .field("width", &gst::IntRange::<i32>::new(0, i32::MAX))
            // .field("height", &gst::IntRange::<i32>::new(0, i32::MAX))
            // .field(
            //     "framerate",
            //     &gst::FractionRange::new(gst::Fraction::new(0, 1), gst::Fraction::new(i32::MAX, 1)),
            // )
            // TODO: framerate/width/height fields are optional?
            .build();

        gst::gst_debug!(CAT, "Using caps {} for input and output", caps);

        let src_pad_template = gst::PadTemplate::new(
            "src",
            gst::PadDirection::Src,
            gst::PadPresence::Always,
            &caps,
        )
        .unwrap();

        gst::gst_debug!(CAT, "Src pad template {:?}", &src_pad_template,);
        klass.add_pad_template(src_pad_template);

        let sink_pad_template = gst::PadTemplate::new(
            "sink",
            gst::PadDirection::Sink,
            gst::PadPresence::Always,
            &caps,
        )
        .unwrap();

        gst::gst_debug!(CAT, "Sink pad template {:?}", &sink_pad_template);
        klass.add_pad_template(sink_pad_template);
    }
}

impl ObjectImpl for GlLcms {}
impl ElementImpl for GlLcms {}

impl BaseTransformImpl for GlLcms {
    fn set_caps(
        &self,
        element: &gst_base::BaseTransform,
        incaps: &gst::Caps,
        outcaps: &gst::Caps,
    ) -> Result<(), gst::LoggableError> {
        let in_info = gst_video::VideoInfo::from_caps(incaps)
            // TODO: Include extra info about incaps/outcaps source?
            .map_err(|e| gst::LoggableError::new(CAT.clone(), e))?;

        let out_info = gst_video::VideoInfo::from_caps(outcaps)
            .map_err(|e| gst::LoggableError::new(CAT.clone(), e))?;

        gst::gst_debug!(
            CAT,
            obj: element,
            "Configured for caps {} to {}",
            incaps,
            outcaps
        );

        *self.state.lock().unwrap() = Some(State { in_info, out_info });

        Ok(())
    }

    fn stop(&self, element: &gst_base::BaseTransform) -> Result<(), gst::ErrorMessage> {
        // Drop state
        let _ = self.state.lock().unwrap().take();

        gst::gst_info!(CAT, obj: element, "Stopped");

        Ok(())
    }

    fn get_unit_size(&self, _element: &gst_base::BaseTransform, caps: &gst::Caps) -> Option<usize> {
        // TODO: This hides any error!
        gst_video::VideoInfo::from_caps(caps)
            .map(|info| info.size())
            .ok()
    }

    // TODO: Don't need filter_caps yet, I think (since input and output are identical, afaik the base implementation handles that)
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        GlLcms::NAME,
        gst::Rank::None,
        GlLcms::get_type(),
    )
}
