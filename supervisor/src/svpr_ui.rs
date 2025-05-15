use std::cmp::PartialEq;
use egui::{Color32, RichText, Vec2};

#[derive(Eq, PartialEq)]
enum MainPageType {
    PageOverView = 0,
    PageProfileServer,
    PageRelayServer,
    PageSettings,
}

pub struct SpvrUI {
    version_code: i32,
    version_name: String,

    main_page: MainPageType,
}

impl Default for SpvrUI {
    fn default() -> Self {
        SpvrUI {
            version_code: 0,
            version_name: "".to_string(),
            main_page: MainPageType::PageOverView,
        }
    }
}

impl eframe::App for SpvrUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            // left
            ui.vertical_centered(|ui| {
                ui.add_space(20.);
                // logo
                ui.add( egui::Image::new(egui::include_image!("../assets/bc_icon.png"))
                            .fit_to_exact_size(egui::Vec2::new(60.0, 60.0))
                            .corner_radius(10),);

                ui.add_space(20.);
                if ui.add_sized([150.0, 32.0], egui::SelectableLabel::new(self.main_page == MainPageType::PageOverView, "Overview")).clicked() {
                    self.main_page = MainPageType::PageOverView;
                }

                ui.add_space(15.);
                if ui.add_sized([150.0, 32.0], egui::SelectableLabel::new(self.main_page == MainPageType::PageProfileServer, "Profile Server")).clicked() {
                    self.main_page = MainPageType::PageProfileServer;
                }

                ui.add_space(15.);
                if ui.add_sized([150.0, 32.0], egui::SelectableLabel::new(self.main_page == MainPageType::PageRelayServer, "Relay Server")).clicked() {
                    self.main_page = MainPageType::PageRelayServer;
                }

                ui.add_space(15.);
                if ui.add_sized([150.0, 32.0], egui::SelectableLabel::new(self.main_page == MainPageType::PageSettings, "Settings")).clicked() {
                    self.main_page = MainPageType::PageSettings
                }

                ui.horizontal(|ui| {
                    ui.add_space(10.);

                    ui.add_space(10.);
                });

            });


        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.main_page == MainPageType::PageOverView {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("HEADING");
                });
            }
            else if self.main_page == MainPageType::PageProfileServer {
                if ui.add_sized([150.0, 32.0], egui::Button::new("Profile Server")).clicked() {

                }
            }
            else if self.main_page == MainPageType::PageRelayServer {
                if ui.add_sized([150.0, 32.0], egui::Button::new("Relay Server")).clicked() {

                }
            }
            else if self.main_page == MainPageType::PageSettings {
                if ui.add_sized([150.0, 32.0], egui::Button::new("Settings")).clicked() {

                }
            }

        });
    }
}