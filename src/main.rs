#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
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

// duplicate cateria is: filename + filesize
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct UniqueFileInfo {
    file_name: String,
    file_len: u64,
}

struct FileSizeCriteria<'a> (&'a str, u64);

const K: u64 = 1024;
const M: u64 = K*K;

const FILESIZE_100M: FileSizeCriteria = FileSizeCriteria("100M", 100*M);
const FILESIZE_10M: FileSizeCriteria = FileSizeCriteria("10M", 10*M);
const FILESIZE_1M: FileSizeCriteria = FileSizeCriteria("1M", 1*M);
const FILESIZE_100K: FileSizeCriteria = FileSizeCriteria("100K", 100*K);
const FILESIZE_10K: FileSizeCriteria = FileSizeCriteria("10K", 10*K);
const FILESIZE_1K: FileSizeCriteria = FileSizeCriteria("1K", 1*K);
const FILESIZE_0: FileSizeCriteria = FileSizeCriteria("0", 0);
const FILESIZES: [FileSizeCriteria; 7] = [ FILESIZE_100M,
                                            FILESIZE_10M,
                                            FILESIZE_1M,
                                            FILESIZE_100K,
                                            FILESIZE_10K,
                                            FILESIZE_1K,
                                            FILESIZE_0 ];

struct MyApp {
    filesize_criteria: u64,
    picked_paths: Vec<String>,
    hashed_files: HashMap<UniqueFileInfo, String>,
    duplicate_files: HashMap<UniqueFileInfo, Vec<(String, bool)>>, // bool value for whether selected for remove
    file_selected: Option<UniqueFileInfo>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            filesize_criteria: FILESIZE_100M.1,
            picked_paths: vec![],
            hashed_files: HashMap::new(),
            duplicate_files: HashMap::new(),
            file_selected: None,
        }
    }

    fn search_duplicate_files(&mut self) {
        // clean data first
        self.duplicate_files.clear();
        self.hashed_files.clear();

        let picked_paths = self.picked_paths.clone();
        for path_str in picked_paths {
            let path = Path::new(&path_str);
            self.visit_dirs(&path, MyApp::handle_onefile).unwrap();
        }
        self.hashed_files.clear();
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

    fn handle_onefile(&mut self, entry: &DirEntry) {
        let file_name = entry.file_name().into_string().unwrap();
        let file_len = entry.metadata().unwrap().len();
        if file_len < self.filesize_criteria { return }
        let file_info = UniqueFileInfo { file_name, file_len };
        if let Some(exists_path) = self.hashed_files.get(&file_info) {
            // duplicated!
            if let Some(files) = self.duplicate_files.get_mut(&file_info) {
                // already duplicated more than 2 times
                files.push((entry.path().to_str().unwrap().to_string(), false));
            } else {
                // new duplicate
                self.duplicate_files.insert(file_info, vec![(exists_path.to_string(), false),
                    (entry.path().to_str().unwrap().to_string(), false)]
                );
            }
        } else {
            // record file in hash
            self.hashed_files.insert(file_info, entry.path().to_str().unwrap().to_string());
        }
    }

    fn remove_selected_files(&mut self) -> io::Result<()> {
        for (_, files) in &mut self.duplicate_files {
            for (file, _) in files.iter().filter(|x| x.1) {
                fs::remove_file(file)?;
            }
            // removed files should also removed from duplicate list
            files.retain(|x| x.1==false);
        }
        // if duplicate file list remains less than 2, it shoul be not duplicate anymore
        self.duplicate_files.retain(|_,v| v.len()>1);
        Ok(())
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application in 中文");

            ui.separator();
            if ui.button("Select directories…").clicked() {
                if let Some(paths) = rfd::FileDialog::new().pick_folders() {
                    self.picked_paths = paths.iter().map(|x| x.display().to_string()).collect();
                }
            }

            ui.label("Picked paths:");
            for path in &self.picked_paths {
                ui.monospace(path);
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Search file size larger than:");
                for filesize in FILESIZES {
                    ui.selectable_value(&mut self.filesize_criteria, filesize.1, filesize.0);
                }
            });

            ui.separator();
            if ui.button("Do search").clicked() {
                self.search_duplicate_files();
            }

            ui.horizontal(|ui| {
            // {
                ui.vertical(|ui| {
                    ui.heading("Duplicated files:");
                    egui::ScrollArea::vertical().id_source("filename")
                        .min_scrolled_height(500.0).max_height(500.0).show(ui, |ui| {
                        let mut file_ref = None;
                        for file_info in self.duplicate_files.keys() {
                            ui.selectable_value(&mut file_ref, Some(file_info),
                                format!("{}\t{}",file_info.file_name, file_info.file_len));
                        }
                        if let Some(file_info) = file_ref {
                            self.file_selected = Some(file_info.clone());
                        }
                    });
                });
                ui.vertical(|ui| {
                    ui.heading("Absolute file path listed, and you can choose to remove:");
                    egui::ScrollArea::vertical().id_source("abs_path")
                        .min_scrolled_height(500.0).max_height(500.0).show(ui, |ui| {
                        if let Some(file_info) = &self.file_selected {
                            if let Some(files) = self.duplicate_files.get_mut(file_info) {
                            // for (_, files) in &mut self.duplicate_files {
                                for (file, selected) in files {
                                    ui.toggle_value(selected, &*file);
                                }
                            }
                        }
                    });
                });
            // }
            });

            ui.add_space(4.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.heading("Files selected for removing:");
                if ui.button("Remove").clicked() {
                    if let Err(_) = self.remove_selected_files() {
                        // delete files error
                        rfd::MessageDialog::new()
                            .set_description("Delete file error!")
                            .set_buttons(rfd::MessageButtons::Ok)
                            .show();
                    }
                }
            });

            for (_, files) in self.duplicate_files.iter() {
                for (file, _) in files.iter().filter(|x| x.1) {
                    ui.label(file);
                }
            }
        });
    }
}
