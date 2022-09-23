#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

use std::io;
// use std::rc::Rc;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::ffi::OsString;
use std::collections::HashMap;

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "simfang".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "C:/Windows/Fonts/simfang.ttf"
        )),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "simfang".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("simfang".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Duplicate Files Cleaner",
        options,
        Box::new(|_cc| Box::new(MyApp::new(_cc))),
    );
}

struct MyApp {
    picked_path: Option<String>,
    filenames: HashMap<OsString, OsString>,
    duplicate_files: Vec<(String, bool)>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            // picked_path: "D:/冯钢/学习资料/Python".to_owned(),
            picked_path: None,
            filenames: HashMap::new(),
            duplicate_files: vec![],
        }
    }

    fn search_duplicate_files(&mut self) {
        // clean data first
        self.duplicate_files.clear();
        self.filenames.clear();

        if let Some(picked_path) = &self.picked_path {
            let path_str = &picked_path.clone();
            let path = Path::new(&path_str);
            self.visit_dirs(&path, MyApp::insert_filename).unwrap();
        }
    }

    // one possible implementation of walking a directory only visiting files
    fn visit_dirs(&mut self, dir: &Path, cb: fn(&mut Self, &DirEntry)) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.visit_dirs(&path, cb)?;
                } else {
                    cb(self, &entry);
                }
            }
        }
        Ok(())
    }

    fn insert_filename(&mut self, entry: &DirEntry) {
        let entries_of_onefile = self.filenames.get(&entry.file_name());
        if let Some(entries_of_onefile) = entries_of_onefile {
            let old_length = fs::metadata(entries_of_onefile).unwrap().len();
            let new_length = entry.metadata().unwrap().len();
            if old_length == new_length {
                // println!("{:?}\t{}", entries_of_onefile, old_length);
                // println!("{:?}\t{}", entry.path(), new_length);
                self.duplicate_files.append(&mut vec![
                    (String::from(entries_of_onefile.to_str().unwrap()),false),
                    (String::from(entry.path().to_str().unwrap()), false)]);
            }
        } else {
            self.filenames.insert(entry.file_name(), entry.path().into_os_string());
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application in 中文");

            if ui.button("Select directories…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.picked_path = Some(path.display().to_string());
                }
            }

            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }
            // ui.label("Directories for searching:");
            // ui.text_edit_singleline(&mut self.picked_path);
            if ui.button("Do search").clicked() {
                self.search_duplicate_files();
            }

            ui.heading("Duplicated files found, and you can choose to remove:");
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for (file, selected) in &mut self.duplicate_files {
                    ui.toggle_value(selected, &*file);
                }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.heading("Files selected for removing:");
                if ui.button("Remove").clicked() {
                }
            });
            // egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                // for (file, _) in self.duplicate_files.iter().filter(|x| x.1) {
                //     ui.label(file);
                // }
            // });
            for (file, _) in self.duplicate_files.iter().filter(|x| x.1) {
                ui.label(file);
            }
        });
    }
}
