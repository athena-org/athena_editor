// Copyright 2015 The Athena Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate phosphorus;
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

use std::env;
use std::cell::RefCell;
use std::rc::Rc;
use gfx::traits::*;
use phosphorus::Gui;
use phosphorus::widget::{ButtonBuilder, Layout, LayoutBuilder, TextBuilder};

struct CancelationToken {
    canceled: bool
}

fn main() {
    // This is all a bunch of placeholder code that needs to be refactored into something coherent

    // Get the target path to open the editor at
    let path = match env::args().nth(1) {
        Some(v) => v,
        None => {
            display_error("No path given!");
            return;
        }
    };

    // Set up our Phosphorus UI
    let token = Rc::new(RefCell::new(CancelationToken { canceled: false }));
    let layout = LayoutBuilder::<gfx_device_gl::Resources>::new()
        .with_background_color([21, 23, 24])
        .with_widget(TextBuilder::new()
            .with_text(&format!("Opened: {}!", path))
            .build_boxed())
        .build();

    display_gui(layout, token);
}

fn display_error(text: &str) {
    let token = Rc::new(RefCell::new(CancelationToken { canceled: false }));
    let token_clone = token.clone();
    let layout = LayoutBuilder::<gfx_device_gl::Resources>::new()
        .with_background_color([21, 23, 24])
        .with_widget(TextBuilder::new()
            .with_text(text)
            .build_boxed())
        .with_widget(ButtonBuilder::new()
            .with_text("Ok")
            .with_callback(Box::new(move || token_clone.borrow_mut().canceled = true))
            .build_boxed())
        .build();

    display_gui(layout, token);
}

fn display_gui(layout: Layout<gfx_device_gl::Resources>, token: Rc<RefCell<CancelationToken>>)
{
    // Set up our window
    let (mut stream, mut device, mut factory) = {
        let window = glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(600, 500)
            .with_title(String::from("Phosphorus Widgets"))
            .build_strict().unwrap();
        gfx_window_glutin::init(window)
    };

    let mut gui = Gui::new(&mut device, layout, |d: &mut gfx_device_gl::Device| d.spawn_factory());

    // Run our actual UI loop
    'main: loop {
        if token.borrow().canceled {
            break 'main;
        }

        // Quit when the window is closed
        for event in stream.out.window.poll_events() {
            match event {
                glutin::Event::Closed => break 'main,
                glutin::Event::MouseMoved((x, y)) =>
                    gui.raise_event(&stream, phosphorus::Event::MouseMoved([x, y])),
                glutin::Event::MouseInput(glutin::ElementState::Released, _) =>
                    gui.raise_event(&stream, phosphorus::Event::MouseClick),
                _ => (),
            }
        }

        stream.clear(gfx::ClearData {color: [1.0, 1.0, 1.0, 1.0], depth: 1.0, stencil: 0});

        // Render our actual GUI
        gui.render(&mut factory, &mut stream);

        // Show the rendered to buffer on the screen
        //stream.present(&mut device); ICE!
        stream.flush(&mut device);
        stream.out.window.swap_buffers();
        device.cleanup();
    }
}
