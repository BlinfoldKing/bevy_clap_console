pub mod errors;

pub use bevy_clap_console_derive::*;
pub use clap;
pub use errors::ConsoleError;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use clap::Parser;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

use std::fmt::Debug;

use bevy::ecs::event::Event;
use bevy_egui::{
    egui::{Key, RichText, ScrollArea, TextEdit, TextStyle, Window},
    EguiContext,
};

pub struct ConsoleDebugPlugin;

pub trait AddCommand {
    fn add_command<T: ConsoleCommand>(&mut self) -> &mut Self;
}

impl AddCommand for App {
    fn add_command<T: ConsoleCommand>(&mut self) -> &mut Self {
        self.add_system(
            console_parse_input
                .pipe(console_handle_input::<T>)
                .pipe(console_handle_error),
        )
        .add_event::<T>();
        self
    }
}

impl Plugin for ConsoleDebugPlugin {
    fn build(&self, app: &mut App) {
        if !app.world.contains_resource::<EguiContext>() {
            app.add_plugin(EguiPlugin);
        }

        app.add_system(console_ui)
            .add_event::<ConsoleOutputEvent>()
            .add_event::<ConsoleInputEvent>();
    }
}

#[derive(Default)]
pub struct ConsoleState {
    input: String,
    output: String,
}

#[derive(Default, Clone)]
pub struct ConsoleOutputEvent(String);

#[derive(Default, Clone)]
pub struct ConsoleInputEvent(String);

fn console_ui(
    mut ctx: ResMut<EguiContext>,
    mut state: Local<ConsoleState>,
    mut out: EventReader<ConsoleOutputEvent>,
    mut emit: EventWriter<ConsoleInputEvent>,
) {
    if out.len() > 0 && state.output.len() > 0 {
        state.output.push_str("\n");
    }
    state.output.push_str(
        &out.iter()
            .map(|x| x.0.clone())
            .collect::<Vec<String>>()
            .join("\n"),
    );
    Window::new("Console")
        // .vscroll(true)
        .default_pos([0.0, 0.0])
        .default_size([500.0, 300.0])
        .resizable(true)
        .show(ctx.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                let scroll_height = ui.available_height() - 30.0;

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .max_height(scroll_height)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for line in state.output.split("\n") {
                                ui.label(RichText::new(line).monospace());
                            }
                        })
                    });
            });

            ui.separator();

            let input = TextEdit::singleline(&mut state.input)
                .desired_width(f32::INFINITY)
                .lock_focus(true)
                .font(TextStyle::Monospace);

            if ui.add(input).lost_focus() && ui.input().key_pressed(Key::Enter) {
                emit.send(ConsoleInputEvent(state.input.clone()));
                state.input = "".to_owned();
            }
        });
}
type Params = Result<Vec<String>, ConsoleError>;

fn console_parse_input(
    mut input: EventReader<ConsoleInputEvent>,
    mut output: EventWriter<ConsoleOutputEvent>,
) -> Params {
    if let Some(input) = input.iter().last() {
        output.send(ConsoleOutputEvent(format!("$ {}", input.0)));
        match shellwords::split(&input.0) {
            Ok(res) => Ok(res),
            Err(_) => Err(ConsoleError::MismatchQuotes),
        }
    } else {
        Ok(vec![])
    }
}

fn console_handle_input<T: ConsoleCommand>(
    In(command): In<Params>,
    mut handle: EventWriter<T>,
) -> Params {
    match command {
        Ok(input) => {
            if input.len() == 0 {
                return Ok(input);
            }
            match T::get(input.clone()) {
                Ok(msg) => {
                    handle.send(msg);
                    Ok(vec![])
                }
                Err(err) => Err(err),
            }
        }
        err => err,
    }
}

fn console_handle_error(In(command): In<Params>, mut output: EventWriter<ConsoleOutputEvent>) {
    if let Err(err) = command {
        output.send(ConsoleOutputEvent(format!(">> {}", err)));
    }
}

pub trait ConsoleCommand
where
    Self: Event + Parser + Debug,
{
    fn get(line: Vec<String>) -> Result<Self, ConsoleError>;
}
