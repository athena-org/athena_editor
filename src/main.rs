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

#![feature(path_ext)]

extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate phosphorus;
extern crate rustc_serialize;

use std::cell::RefCell;
use std::env;
use std::fs::{File, PathExt};
use std::io::{Read, Write};
use std::path::{PathBuf};
use std::rc::Rc;
use gfx::traits::*;
use phosphorus::Gui;
use phosphorus::widget::{ButtonBuilder, Layout, LayoutBuilder, TextBuilder};
use rustc_serialize::json;

struct SharedData {
    canceled: bool,
    queued_layout: Option<Layout<gfx_device_gl::Resources>>
}

#[derive(RustcDecodable, RustcEncodable, Clone)]
struct EntityEntry {
    x: f32,
    y: f32,
    z: f32
}

#[derive(RustcDecodable, RustcEncodable, Clone)]
struct WorldModel {
    entities: Vec<Rc<RefCell<EntityEntry>>>
}

#[derive(RustcDecodable, RustcEncodable, Clone)]
struct Model {
    current_entity: Option<Rc<RefCell<EntityEntry>>>,
    worlds: Vec<WorldModel>
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

    // Make sure it's a zeus project
    let mut toml_path = PathBuf::from(path.clone());
    toml_path.push("Zeus.toml");
    if !toml_path.exists() { display_error("Not a zeus project!"); return; }

    // Get the actual path of the data file
    let mut editor_path = PathBuf::from(path);
    editor_path.push("editor.json");

    let model = Rc::new(RefCell::new(if !editor_path.exists() {
        println!("Creating new editor.json...");
        let model = Model { current_entity: None, worlds: vec![] };

        save_model(&editor_path, &model);

        model
    } else {
        println!("Loading in editor.json...");
        let mut file = File::open(editor_path.clone()).unwrap();
        let mut file_data = String::new();
        file.read_to_string(&mut file_data).unwrap();

        json::decode::<Model>(&file_data).unwrap()
    }));
    model.borrow_mut().current_entity = None;

    // Set up our Phosphorus UI
    let data = Rc::new(RefCell::new(SharedData { canceled: false, queued_layout: None }));
    let layout = generate_view(editor_path, data.clone(), model);

    data.borrow_mut().queued_layout = Some(layout);

    display_gui(data);
}

fn save_model(proj_path: &PathBuf, model: &Model){
    let mut file = match File::create(proj_path) {
        Ok(f) => f,
        Err(e) => panic!(format!("Folder does not exist or is not accessable, error: {:?}", e))
    };
    file.write_all(&json::encode(&model).unwrap().as_bytes()).unwrap();
}

fn generate_view(proj_path: PathBuf, data: Rc<RefCell<SharedData>>, model: Rc<RefCell<Model>>) -> Layout<gfx_device_gl::Resources> {
    // Save our model before generating so the file's always up-to-date with what's visible
    save_model(&proj_path, &model.borrow());

    let mut builder = LayoutBuilder::<gfx_device_gl::Resources>::new()
        .with_background_color([21, 23, 24]);

    // Show all of our worlds
    let mut wnum = 0;
    for world in &model.borrow_mut().worlds {
        builder = builder
            .with_widget(TextBuilder::new()
                .with_text(&format!("== World #{} ==", wnum))
                .build_boxed());

        // Show all of our entities
        let mut ennum = 0;
        for rc_entity in &world.entities {
            let entity = rc_entity.borrow_mut();
            let tmp_rc_entity = rc_entity.clone();
            let tmp_model = model.clone();
            let tmp_data = data.clone();
            let tmp_path = proj_path.clone();

            builder = builder
                .with_widget(ButtonBuilder::new()
                    .with_text(&format!("Entity #{} - {{x: {}, y: {}, z: {}}}", ennum, entity.x, entity.y, entity.z))
                    .with_size([180, 20])
                    .with_callback(Box::new(move || {
                        {
                            let mut m = tmp_model.borrow_mut();
                            m.current_entity = Some(tmp_rc_entity.clone());
                        }

                        tmp_data.borrow_mut().queued_layout = Some(generate_view(tmp_path.clone(), tmp_data.clone(), tmp_model.clone()));
                    }))
                    .build_boxed());
            ennum += 1;
        }

        // Show a button to create a new entity
        let tmp_model = model.clone();
        let tmp_data = data.clone();
        let tmp_path = proj_path.clone();
        builder = builder
            .with_widget(ButtonBuilder::new()
                .with_text("Add Entity")
                .with_callback(Box::new(move || {
                    {
                        let mut m = tmp_model.borrow_mut();
                        let w = m.worlds.get_mut(wnum).unwrap();
                        w.entities.push(Rc::new(RefCell::new(EntityEntry { x: 0.0, y: 0.0, z: 0.0 })));
                    }

                    tmp_data.borrow_mut().queued_layout = Some(generate_view(tmp_path.clone(), tmp_data.clone(), tmp_model.clone()));
                }))
                .build_boxed()
            );
        wnum += 1;

        // Placeholder text as padding
        builder = builder.with_widget(TextBuilder::new().build_boxed());
    }

    // Show an add world button
    let tmp_model = model.clone();
    let tmp_data = data.clone();
    let tmp_path = proj_path.clone();
    builder = builder
        .with_widget(ButtonBuilder::new()
            .with_text("Add World")
            .with_callback(Box::new(move || {
                {
                    let mut m = tmp_model.borrow_mut();
                    m.worlds.push(WorldModel { entities: vec![] });
                }

                tmp_data.borrow_mut().queued_layout = Some(generate_view(tmp_path.clone(), tmp_data.clone(), tmp_model.clone()));
            }))
            .build_boxed());

    // Show an editor for the currently selected entity
    builder = builder
        .with_widget(TextBuilder::new().build_boxed())
        .with_widget(TextBuilder::new()
            .with_text("=== Edit Entity ===")
            .build_boxed()
        );

    {
        let m = model.borrow_mut();
        if let &Some(ref rc_entity) = &m.current_entity {
            let entity = rc_entity.borrow();

            builder = builder
                .with_widget(TextBuilder::new()
                    .with_text(&format!("X: {} Y: {} Z: {}", entity.x, entity.y, entity.z))
                    .build_boxed()
                );

            // Show buttons for changing position
            builder = show_adder_button(builder, &proj_path, &data, &model, rc_entity, "X + 1", Box::new(|e| e.x += 1.0));
            builder = show_adder_button(builder, &proj_path, &data, &model, rc_entity, "X - 1", Box::new(|e| e.x -= 1.0));
            builder = show_adder_button(builder, &proj_path, &data, &model, rc_entity, "Y + 1", Box::new(|e| e.y += 1.0));
            builder = show_adder_button(builder, &proj_path, &data, &model, rc_entity, "Y - 1", Box::new(|e| e.y -= 1.0));
            builder = show_adder_button(builder, &proj_path, &data, &model, rc_entity, "Z + 1", Box::new(|e| e.z += 1.0));
            builder = show_adder_button(builder, &proj_path, &data, &model, rc_entity, "Z - 1", Box::new(|e| e.z -= 1.0));
        }
    }

    builder.build()
}

fn show_adder_button(
    mut builder: LayoutBuilder<gfx_device_gl::Resources>,
    path: &PathBuf, data: &Rc<RefCell<SharedData>>, model: &Rc<RefCell<Model>>, rc_entity: &Rc<RefCell<EntityEntry>>,
    text: &'static str, adder: Box<Fn(&mut EntityEntry)>)
 -> LayoutBuilder<gfx_device_gl::Resources>
{
    let tmp_model = model.clone();
    let tmp_data = data.clone();
    let tmp_path = path.clone();
    let tmp_entity = rc_entity.clone();
    builder = builder
        .with_widget(ButtonBuilder::new()
            .with_text(text)
            .with_callback(Box::new(move || {
                {
                    let mut entity = tmp_entity.borrow_mut();
                    adder(&mut entity);
                }

                tmp_data.borrow_mut().queued_layout = Some(generate_view(tmp_path.clone(), tmp_data.clone(), tmp_model.clone()));
            }))
            .build_boxed()
        );

    builder
}

fn display_error(text: &str) {
    let data = Rc::new(RefCell::new(SharedData { canceled: false, queued_layout: None }));
    let data_clone = data.clone();
    let layout = LayoutBuilder::<gfx_device_gl::Resources>::new()
        .with_background_color([21, 23, 24])
        .with_widget(TextBuilder::new()
            .with_text(text)
            .build_boxed())
        .with_widget(ButtonBuilder::new()
            .with_text("Ok")
            .with_callback(Box::new(move || data_clone.borrow_mut().canceled = true))
            .build_boxed())
        .build();

    data.borrow_mut().queued_layout = Some(layout);

    display_gui(data);
}

fn display_gui(data: Rc<RefCell<SharedData>>)
{
    // Set up our window
    let (mut stream, mut device, mut factory) = {
        let window = glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(600, 500)
            .with_title(String::from("Athena Editor"))
            .build_strict().unwrap();
        gfx_window_glutin::init(window)
    };

    let mut gui = Gui::new(&mut device, &mut factory, data.borrow_mut().queued_layout.take().unwrap());

    // Run our actual UI loop
    'main: loop {
        {
            let mut d = data.borrow_mut();

            if d.canceled {
                break 'main;
            }

            if let Some(layout) = d.queued_layout.take() {
                gui.set_root(layout);
            }
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
