use std::io::Write;

use eframe::egui;
use eframe::epaint::{Pos2, Vec2};
use rand::random;
use tracing::metadata::LevelFilter;
use vst::api::{Events, Supported};
use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::event::Event;
use vst::plugin::{CanDo, Category, Info, Plugin};

#[derive(Default)]
struct Whisper {
    // Added a counter in our plugin struct.
    notes: u8,
}

// We're implementing a trait `Plugin` that does all the VST-y stuff for us.
impl Plugin for Whisper {
    fn get_info(&self) -> Info {
        Info {
            name: "Whisper".to_string(),

            // Used by hosts to differentiate between plugins.
            unique_id: 1337,

            // We don't need inputs
            inputs: 0,

            // We do need two outputs though.  This is default, but let's be
            // explicit anyways.
            outputs: 2,

            // Set our category
            category: Category::Synth,

            // We don't care about other stuff, and it can stay default.
            ..Default::default()
        }
    }

    // Here's the function that allows us to receive events
    fn process_events(&mut self, events: &Events) {
        // Some events aren't MIDI events - so let's do a match
        // to make sure we only get MIDI, since that's all we care about.
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    // Check if it's a noteon or noteoff event.
                    // This is difficult to explain without knowing how the MIDI standard works.
                    // Basically, the first byte of data tells us if this signal is a note on event
                    // or a note off event.  You can read more about that here:
                    // https://www.midi.org/specifications/item/table-1-summary-of-midi-message
                    match ev.data[0] {
                        // if note on, increment our counter
                        144 => self.notes += 1u8,

                        // if note off, decrement our counter
                        128 => self.notes -= 1u8,
                        _ => (),
                    }
                    // if we cared about the pitch of the note, it's stored in `ev.data[1]`.
                }
                // We don't care if we get any other type of event
                _ => (),
            }
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        // `buffer.split()` gives us a tuple containing the
        // input and output buffers.  We only care about the
        // output, so we can ignore the input by using `_`.
        let (_, mut output_buffer) = buffer.split();

        // We only want to process *anything* if a note is being held.
        // Else, we can fill the output buffer with silence.
        if self.notes == 0 {
            for output_channel in output_buffer.into_iter() {
                // Let's iterate over every sample in our channel.
                for output_sample in output_channel {
                    *output_sample = 0.0;
                }
            }
            return;
        }

        // Now, we want to loop over our output channels.  This
        // includes our left and right channels (or more, if you
        // are working with surround sound).
        for output_channel in output_buffer.into_iter() {
            // Let's iterate over every sample in our channel.
            for output_sample in output_channel {
                // For every sample, we want to generate a random value
                // from -1.0 to 1.0.
                *output_sample = (random::<f32>() - 0.5f32) * 2f32;
            }
        }
    }

    // It's good to tell our host what our plugin can do.
    // Some VST hosts might not send any midi events to our plugin
    // if we don't explicitly tell them that the plugin can handle them.
    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            // Tell our host that the plugin supports receiving MIDI messages
            CanDo::ReceiveMidiEvent => Supported::Yes,
            // Maybe it also supports ather things
            _ => Supported::Maybe,
        }
    }

    fn new(host: vst::prelude::HostCallback) -> Self
    where
        Self: Sized,
    {
        let file = std::fs::File::create(
            "C:/Users/hato2/Desktop/vst-rs-example-egui/target/debug/log.txt",
        )
        .unwrap();
        tracing_subscriber::fmt()
            .with_file(true)
            .with_max_level(LevelFilter::TRACE)
            .with_writer(std::sync::Arc::new(file))
            .init();
        Whisper { notes: 0 }
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        Some(Box::new(VstGui::default()))
    }
}

vst::plugin_main!(Whisper);

#[derive(Default)]
struct VstGui(Option<eframe::Idle>);

impl VstGui {
    fn close(&mut self) {
        if let Some(mut idle) = self.0.take() {
            tracing::debug!("close");
            idle.close();
        }
    }
}

impl Editor for VstGui {
    fn size(&self) -> (i32, i32) {
        (640, 480)
    }

    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn open(&mut self, parent: *mut std::os::raw::c_void) -> bool {
        self.close();

        let mut options = eframe::NativeOptions::default();
        options.parent_window = Some(parent as _);
        options.decorated = false;
        options.resizable = false;

        let idle = eframe::idle_native(
            "My egui App",
            options,
            Box::new(|_cc| Box::new(MyApp::default())),
        );

        self.0 = Some(idle);

        true
    }

    fn idle(&mut self) {
        let mut exit = false;
        if let Some(idle) = self.0.as_mut() {
            tracing::debug!("idle start");
            exit = idle.idle();
            tracing::debug!("idle end");
        }
        if exit {
            self.0 = None;
        }
    }

    fn close(&mut self) {
        self.close();
    }

    fn is_open(&mut self) -> bool {
        self.0.is_some()
    }
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}
