#![windows_subsystem = "windows"]

mod compiler;

pub use compiler::{evaluater, tokeniser};

use eframe::{egui::*, NativeOptions};
use clipboard::{ClipboardContext, ClipboardProvider};

#[derive(Default, Clone)]
pub struct MyApp {
    text: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("2025/04/13 by Kinokov Shotaskovich");
                ui.label("Evaluater");
                ui.add_sized(ui.available_size() - Vec2 { x: 0., y: 40. }, egui::TextEdit::multiline(&mut self.text));
                if ui.button("Compile then Copy").clicked() {
                    let mut compiler = compiler::Compiler::new();
                    let formula = tokeniser::tokenize(self.text.clone());
                    let result = evaluater::evaluate(&mut compiler, &formula).join("\n");
                    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                    ctx.set_contents(result.to_owned()).unwrap();
                }
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