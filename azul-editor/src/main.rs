extern crate azul;

use azul::prelude::*;
use azul::widgets::label::Label;
use azul::widgets::button::Button;


pub struct SceneView {

}

#[derive(Default)]
pub struct SceneRenderer {}

impl SceneRenderer {
	pub fn dom<T: Layout>(data: &SceneView) -> Dom<T> {
		let ptr = StackCheckedPointer::new(data);
		Dom::new(NodeType::GlTexture(GlCallback(Self::render), ptr))
	}

	fn render<T: Layout>(
        glInfo: GlCallbackInfoUnchecked<T>) -> Option<Texture>

    {

		fn render_inner(component_state: &mut SceneView, info: LayoutInfo<T>, bounds: HidpiAdjustedBounds) -> Texture {
             let texture = info.window.create_texture(width as u32, height as u32);


             // You could update your component_state here, if you'd like.
             texture
        }

        // Cast the StackCheckedPointer to a CubeControl, then invoke the render_inner function on it.
        Some(unsafe { state.invoke_mut_texture(render_inner, glInfo.ptr, glInfo.layout_info, glInfo.bounds) })

	}
}

struct CounterApplication {
	counter: usize,
}


impl Layout for CounterApplication {
    fn layout(&self, _: LayoutInfo<Self>) -> Dom<Self> {
        let label = Label::new(format!("{}", self.counter)).dom();
        let button = Button::with_label("Update counter").dom()
            .with_callback(On::MouseUp, update_counter);

        Dom::div()
            .with_child(label)
            .with_child(button)
    }
}

fn update_counter(event: CallbackInfo<CounterApplication>) -> UpdateScreen {
    event.state.data.counter += 1;
    Redraw
}

fn main() {
    let mut app = App::new(CounterApplication { counter: 0 }, AppConfig::default()).unwrap();

    let window = app.create_window(WindowCreateOptions::default(), css::native()).unwrap();

    app.run(window).unwrap();
}
