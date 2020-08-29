use gst::glib;
use gst::prelude::*;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::*;
use gst_gl::subclass::prelude::*;
use gst_gl::*;
use gstreamer as gst;
use gstreamer_base as gst_base;
use gstreamer_gl as gst_gl;
use gstreamer_video as gst_video;

use glib::subclass;
use glib::subclass::prelude::*;

use once_cell::sync::Lazy;
use std::sync::Mutex;

const GL_FRAGMENT_SHADER: u32 = 0x8B30;

const FRAGMENT_SHADER: &str = r#"
varying vec2 v_texcoord;
uniform sampler2D tex;
void main () {
    vec4 rgba = texture2D (tex, v_texcoord);
    // Test swizzle
    gl_FragColor = rgba.gbra;
}
"#;

struct State {
    shader: GLShader,
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

impl ObjectImpl for GlLcms {}
impl ElementImpl for GlLcms {}
impl BaseTransformImpl for GlLcms {}
impl GLBaseFilterImpl for GlLcms {}

fn create_shader(filter: &GLFilter, context: &GLContext) -> GLShader {
    let shader = GLShader::new(context);
    let version = GLSLVersion::None;
    let profile = GLSLProfile::ES | GLSLProfile::COMPATIBILITY;

    let vertex = GLSLStage::new_default_vertex(context);
    vertex.compile().unwrap();
    shader.attach_unlocked(&vertex).unwrap();

    let shader_parts = [
        &GLShader::string_get_highest_precision(context, version, profile).unwrap(),
        FRAGMENT_SHADER,
    ];

    gst::gst_debug!(
        CAT,
        obj: filter,
        "Compiling fragment shader parts {:?}",
        &shader_parts
    );

    let fragment =
        GLSLStage::with_strings(context, GL_FRAGMENT_SHADER, version, profile, &shader_parts);
    fragment.compile().unwrap();
    shader.attach_unlocked(&fragment).unwrap();
    shader.link().unwrap();

    gst::gst_debug!(CAT, obj: filter, "Successfully linked {:?}", shader);

    shader
}

impl GLFilterImpl for GlLcms {
    fn filter_texture(
        &self,
        filter: &GLFilter,
        input: &mut GLMemory,
        output: &mut GLMemory,
    ) -> bool {
        let context = filter.get_property_context().unwrap();

        let mut state = self.state.lock().unwrap();

        if state.is_none() {
            let shader = create_shader(filter, &context);
            *state = Some(State { shader });
        }
        let State { shader } = state.as_ref().unwrap();

        filter.render_to_target_with_shader(input, output, shader);

        gst::gst_trace!(CAT, obj: filter, "Render finished");

        true
    }
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        GlLcms::NAME,
        gst::Rank::None,
        GlLcms::get_type(),
    )
}
