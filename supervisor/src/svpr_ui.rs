use std::cmp::PartialEq;
use egui::{Color32, Label, RichText, Vec2};
use egui::WidgetType::TextEdit;

#[derive(Eq, PartialEq)]
enum MainPageType {
    PageServerSettings = 0,
    PageStatistics,
    PageRelayServer,
    PageSettings,
}

pub struct SpvrUI {
    version_code: i32,
    version_name: String,

    spvr_server_name: String,
    spvr_tcp_port: String,
    spvr_single_deploy: bool,

    relay_server_name: String,
    relay_tcp_port: String,
    relay_grpc_port: String,
    relay_spvr_ip: String,
    relay_redis_path: String,

    pr_server_name: String,
    pr_tcp_port: String,
    pr_grpc_port: String,
    pr_spvr_ip: String,
    pr_mongodb_path: String,

    main_page: MainPageType,
}

impl Default for SpvrUI {
    fn default() -> Self {
        SpvrUI {
            version_code: 0,
            version_name: "".to_string(),
            spvr_server_name: "Srv.Supervisor.01".to_string(),
            spvr_tcp_port: "30500".to_string(),
            spvr_single_deploy: true,

            relay_server_name: "Srv.Relay.01".to_string(),
            relay_tcp_port: "30600".to_string(),
            relay_grpc_port: "40600".to_string(),
            relay_spvr_ip: "127.0.0.1".to_string(),
            relay_redis_path: "redis://127.0.0.1:6379/".to_string(),

            pr_server_name: "Srv.Profile.01".to_string(),
            pr_tcp_port: "30700".to_string(),
            pr_grpc_port: "40700".to_string(),
            pr_spvr_ip: "127.0.0.1".to_string(),
            pr_mongodb_path: "mongodb://localhost:27017/".to_string(),

            main_page: MainPageType::PageServerSettings,
        }
    }
}

impl eframe::App for SpvrUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // dark
        ctx.set_visuals(egui::Visuals::dark());

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            // left
            ui.vertical_centered(|ui| {
                ui.add_space(20.);
                let btn_size = [150.0, 32.0];
                // logo
                ui.add( egui::Image::new(egui::include_image!("../assets/bc_icon.png"))
                            .fit_to_exact_size(egui::Vec2::new(60.0, 60.0))
                            .corner_radius(10),);

                ui.add_space(20.);
                if ui.add_sized(btn_size, egui::SelectableLabel::new(self.main_page == MainPageType::PageServerSettings, "Server Settings")).clicked() {
                    self.main_page = MainPageType::PageServerSettings;
                }

                ui.add_space(15.);
                if ui.add_sized(btn_size, egui::SelectableLabel::new(self.main_page == MainPageType::PageStatistics, "Statistics")).clicked() {
                    self.main_page = MainPageType::PageStatistics;
                }

            });


        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.main_page == MainPageType::PageServerSettings {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(15.0);
                    ui.heading("Supervisor Server Settings");
                    ui.add_space(15.0);

                    let lbl_size = [200.0, 20.0];

                    egui::Grid::new("grid_spvr")
                        .num_columns(3)
                        .spacing([40.0, 14.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Server Name:");
                            ui.label(self.spvr_server_name.clone());
                            ui.end_row();

                            ui.label("Document");
                            use egui::special_emojis::GITHUB;
                            ui.hyperlink_to(
                                format!("{GITHUB} Document on GitHub"),
                                "https://github.com/rgaa-software/gammaray",
                            );
                            ui.end_row();

                            ui.label("Listening Port:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.spvr_tcp_port)
                                             .hint_text("Tcp Port..."));
                            ui.label("TCP server will listen on this port");
                            ui.end_row();

                            ui.label("Single Instance Deploy:");
                            ui.checkbox(&mut self.spvr_single_deploy, "");
                            ui.end_row();

                        });

                    ui.add_space(20.0);
                    ui.heading("Relay Server Settings");
                    ui.add_space(15.0);

                    egui::Grid::new("grid_relay")
                        .num_columns(2)
                        .spacing([40.0, 14.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Server Name:");
                            ui.label(self.relay_server_name.clone());
                            ui.end_row();

                            ui.label("Listening Port:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.relay_tcp_port)
                                             .hint_text("Tcp Port..."));
                            ui.end_row();

                            ui.label("GRPC Port:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.relay_grpc_port)
                                             .hint_text("Grpc Port..."));
                            ui.end_row();

                            ui.label("Supervisor IP:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.relay_spvr_ip)
                                             .interactive(false)
                                             .hint_text("Supervisor IP"));
                            ui.end_row();

                            ui.label("Supervisor Listening Port:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.spvr_tcp_port)
                                             .interactive(false)
                                             .hint_text("Supervisor Listening Port"));

                            ui.end_row();

                            ui.label("Redis Path:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.relay_redis_path)
                                             .hint_text("mongodb Path"));

                            ui.end_row();
                        });

                    ui.add_space(20.0);
                    ui.heading("Profile Server Settings");
                    ui.add_space(15.0);

                    egui::Grid::new("grid_pr")
                        .num_columns(2)
                        .spacing([40.0, 14.0])
                        .striped(true)
                        .show(ui, |ui| {

                            ui.label("Server Name:");
                            ui.label(self.pr_server_name.clone());
                            ui.end_row();

                            ui.label("Listening Port:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.pr_tcp_port)
                                             .hint_text("Tcp Port..."));
                            ui.end_row();

                            ui.label("GRPC Port:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.pr_grpc_port)
                                             .hint_text("Grpc Port..."));
                            ui.end_row();

                            ui.label("Supervisor IP:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.pr_spvr_ip)
                                             .interactive(false)
                                             .hint_text("Supervisor IP"));
                            ui.end_row();

                            ui.label("Supervisor Listening Port:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.spvr_tcp_port)
                                             .interactive(false)
                                             .hint_text("Supervisor Listening Port"));

                            ui.end_row();

                            ui.label("MongoDB Path:");
                            ui.add_sized(lbl_size,
                                         egui::TextEdit::singleline(&mut self.pr_mongodb_path)
                                             .interactive(false)
                                             .hint_text("mongodb Path"));

                            ui.end_row();

                        });

                });
            }
            else if self.main_page == MainPageType::PageStatistics {
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