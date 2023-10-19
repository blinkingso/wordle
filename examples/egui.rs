use eframe::egui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Wordle",
        native_options,
        Box::new(|_cc| Box::new(Counter::default())),
    )?;
    Ok(())
}

#[derive(Default)]
struct Counter {
    value: i32,
}

impl eframe::App for Counter {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Ui Header");
            ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    self.value -= 1;
                }
                ui.label(self.value.to_string());
                if ui.button("+").clicked() {
                    self.value += 1;
                }
            });
        });
    }
}
