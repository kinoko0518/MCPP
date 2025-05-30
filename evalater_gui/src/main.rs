#![windows_subsystem = "windows"]

use eframe::{egui::*, NativeOptions};
use clipboard::{ClipboardContext, ClipboardProvider};
use mcpp_core::compiler;

#[derive(Default, Clone)]
pub struct MyApp {
    text: String,
    error: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("2025/04/13 by Kinokov Shotaskovich");
                ui.label("Evaluater");
                ui.add_sized(ui.available_size() - Vec2 { x: 0., y: 40. }, egui::TextEdit::multiline(&mut self.text));
                if ui.button("Compile then Copy").clicked() {
                    match compiler::Compiler::from("MCPP").evaluate(self.text.clone()) {
                        Ok(o) => {
                            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                            ctx.set_contents(o).unwrap();
                        },
                        Err(e) => {self.error = format!("{}", e);}
                    }
                }
                ui.label(
                    RichText::from(self.error.clone())
                        .color(Color32::LIGHT_RED)
                );
            });
        });
    }
}

fn main() {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_min_inner_size([320.0, 240.0])
            .with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../icon.png")).unwrap()),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "FormulaParser",
        options,
        Box::new(|_| {Ok(Box::<MyApp>::default())}),
    );
}