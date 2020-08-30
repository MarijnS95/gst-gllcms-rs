use gst::glib;
use gst::prelude::*;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::*;
use gst_gl::subclass::prelude::*;
use gst_gl::*;
use gstreamer as gst;
use gstreamer_base as gst_base;
use gstreamer_gl as gst_gl;

use glib::subclass;
use glib::subclass::prelude::*;

use gfx_gl as gl;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Default vertex shader from gst_gl_shader_string_vertex_default
const VERTEX_SHADER: &str = r#"
in vec4 a_position;
in vec2 a_texcoord;
out vec2 v_texcoord;
void main()
{
   gl_Position = a_position;
   v_texcoord = a_texcoord;
}"#;

const FRAGMENT_SHADER: &str = r#"
in vec2 v_texcoord;
out vec4 fragColor;

uniform sampler2D tex;

void main () {
    vec4 rgba = texture2D (tex, v_texcoord);
    // Test swizzle
    fragColor = rgba.gbra;
}
"#;

const DEFAULT_BRIGHTNESS: f64 = 0f64;
const DEFAULT_CONTRAST: f64 = 1f64;
const DEFAULT_HUE: f64 = 0f64;
const DEFAULT_SATURATION: f64 = 0f64;

#[derive(Debug, Clone, PartialEq)]
struct Settings {
    icc: Option<String>,
    brightness: f64,
    contrast: f64,
    hue: f64,
    saturation: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            icc: None,
            brightness: DEFAULT_BRIGHTNESS,
            contrast: DEFAULT_CONTRAST,
            hue: DEFAULT_HUE,
            saturation: DEFAULT_SATURATION,
        }
    }
}

struct State {
    shader: GLShader,
}

struct GlLcms {
    // TODO: Need multi-reader lock?
    settings: Mutex<Settings>,
    state: Mutex<Option<State>>,
}

static PROPERTIES: &[subclass::Property] = &[
    subclass::Property("icc", |name| {
        glib::ParamSpec::string(
            name,
            "ICC Profile",
            "Path to ICC color profile",
            None,
            glib::ParamFlags::READWRITE,
        )
    }),
    subclass::Property("brightness", |name| {
        glib::ParamSpec::double(
            name,
            "Bright",
            "Extra brightness correction",
            // TODO: Docs don't clarify min and max!
            -1f64,
            1f64,
            DEFAULT_BRIGHTNESS,
            glib::ParamFlags::READWRITE,
        )
    }),
    subclass::Property("contrast", |name| {
        glib::ParamSpec::double(
            name,
            "Contrast",
            "Extra contrast correction",
            // TODO: Docs don't clarify min and max!
            0f64,
            2f64,
            DEFAULT_CONTRAST,
            glib::ParamFlags::READWRITE,
        )
    }),
    subclass::Property("hue", |name| {
        glib::ParamSpec::double(
            name,
            "Hue",
            "Extra hue displacement in degrees",
            0f64,
            360f64,
            DEFAULT_HUE,
            glib::ParamFlags::READWRITE,
        )
    }),
    subclass::Property("saturation", |name| {
        glib::ParamSpec::double(
            name,
            "Saturation",
            "Extra saturation correction",
            // TODO: Docs don't clarify min and max!
            -1f64,
            1f64,
            DEFAULT_SATURATION,
            glib::ParamFlags::READWRITE,
        )
    }),
    // TODO: Model white balance src+dest as structure
    /*
    subclass::Property("temp", |name| {
        glib::ParamSpec::value_array(
            name,
            "Source temperature",
            "Source white point temperature",
            &glib::ParamSpec::uint(
                name,
                "Source temperature",
                "Source white point temperature",
                // TODO: Docs don't clarify min and max!
                0,
                std::u32::MAX,
                0,
                glib::ParamFlags::READWRITE,
            ),
            glib::ParamFlags::READWRITE,
        )
    }),
    */
];

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
            settings: Mutex::new(Default::default()),
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

        klass.install_properties(&PROPERTIES);

        // klass.configure(
        //     gst_base::subclass::BaseTransformMode::NeverInPlace,
        //     false,
        //     false,
        // );

        GLFilter::add_rgba_pad_templates(klass)
    }
}

impl ObjectImpl for GlLcms {
    fn set_property(&self, obj: &glib::Object, id: usize, value: &glib::Value) {
        let prop = &PROPERTIES[id];
        let element = obj.downcast_ref::<gst_base::BaseTransform>().unwrap();

        gst::gst_info!(CAT, obj: element, "Changing {:?} to {:?}", prop, value);

        let mut settings = self.settings.lock().unwrap();

        match prop.0 {
            "icc" => settings.icc = value.get().expect("Type mismatch"),
            "brightness" => settings.brightness = value.get_some().expect("Type mismatch"),
            "contrast" => settings.contrast = value.get_some().expect("Type mismatch"),
            "hue" => settings.hue = value.get_some().expect("Type mismatch"),
            "saturation" => settings.saturation = value.get_some().expect("Type mismatch"),
            _ => {
                let element = obj.downcast_ref::<gst_base::BaseTransform>().unwrap();
                gst::gst_error!(CAT, obj: element, "Property {} doesn't exist", prop.0);
            }
        }
    }

    fn get_property(&self, obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        let prop = &PROPERTIES[id];
        let settings = self.settings.lock().unwrap();

        match prop.0 {
            "icc" => Ok(settings.icc.to_value()),
            "brightness" => Ok(settings.brightness.to_value()),
            "contrast" => Ok(settings.contrast.to_value()),
            "hue" => Ok(settings.hue.to_value()),
            "saturation" => Ok(settings.saturation.to_value()),
            _ => {
                let element = obj.downcast_ref::<gst_base::BaseTransform>().unwrap();
                gst::gst_error!(CAT, obj: element, "Property {} doesn't exist", prop.0);
                Err(())
            }
        }
    }
}
impl ElementImpl for GlLcms {}
impl BaseTransformImpl for GlLcms {}
impl GLBaseFilterImpl for GlLcms {}

fn create_shader(filter: &GLFilter, context: &GLContext) -> GLShader {
    let shader = GLShader::new(context);
    let version = GLSLVersion::_400;
    let profile = GLSLProfile::empty();

    // let vertex = GLSLStage::new_default_vertex(context);
    // new_default_vertex assumes GLSLVersion::None and ES | COMPATIBILITY profile
    let shader_parts = [
        // TODO: This function is only in my branch of gstreamer-rs!
        &format!(
            "#version {}",
            &GLSLVersion::profile_to_string(version, profile).unwrap()
        ) as &str,
        VERTEX_SHADER,
    ];

    gst::gst_debug!(
        CAT,
        obj: filter,
        "Compiling vertex shader parts {:?}",
        &shader_parts
    );

    let vertex =
        GLSLStage::with_strings(context, gl::VERTEX_SHADER, version, profile, &shader_parts);
    vertex.compile().unwrap();
    shader.attach_unlocked(&vertex).unwrap();

    let shader_parts = [
        // TODO: This function is only in my branch of gstreamer-rs!
        &format!(
            "#version {}",
            &GLSLVersion::profile_to_string(version, profile).unwrap()
        ) as &str,
        FRAGMENT_SHADER,
    ];

    gst::gst_debug!(
        CAT,
        obj: filter,
        "Compiling fragment shader parts {:?}",
        &shader_parts
    );

    let fragment = GLSLStage::with_strings(
        context,
        gl::FRAGMENT_SHADER,
        version,
        profile,
        &shader_parts,
    );
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
