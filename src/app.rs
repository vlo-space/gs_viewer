
use std::{fs, io::{BufRead, BufReader, Read}, sync::{atomic::AtomicBool, Arc, Mutex}, thread, time::{Duration, Instant}};

use crate::{data::{parse_log_line, SensedData}, tabs::{data::{data_tab, DataTabState}, map::{map_tab, MapTabState}, plot::{plot_tab, PlotTabState}}};

pub struct TemplateApp {
    current_tab: Tab,
    data_source: DataSource,

    plot_state: PlotTabState,
    data_state: DataTabState,
    map_state: MapTabState,

    auto_repaint: bool,

    status_message: Option<StatusMessage>
}

#[derive(Debug, Clone)]
enum DataSource {
    File {
        name: String,
        data: Vec<SensedData>
    },
    SerialPort {
        port_name: String,
        baud_rate: u32,

        data: Arc<Mutex<Vec<SensedData>>>,
        cancel_reader: Arc<AtomicBool>
    },
    None
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Data,
    Plot,
    Map
}

#[derive(Debug, Clone)]
struct StatusMessage {
    since: Instant,
    duration: Duration,
    text: String
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Self {
            current_tab: Tab::Data,

            data_source: DataSource::None,

            plot_state: PlotTabState::default(),
            data_state: DataTabState {
                stick_to_bottom: true
            },
            map_state: MapTabState::new(&cc.egui_ctx),
            auto_repaint: false,
            status_message: None
        }
    }
}

fn spawn_data_reader_thread<'a, T: Read + BufRead + Send + 'static>(mut reader: T, data: Arc<Mutex<Vec<SensedData>>>) -> Arc<AtomicBool> {
    let canceller = Arc::new(AtomicBool::new(false));
    let cloned_canceller = canceller.clone();
    thread::spawn(move || {
        loop {
            let mut string = String::new();
            let _ = reader.read_line(&mut string);

            let sensed = parse_log_line(&string);
            
            if canceller.load(std::sync::atomic::Ordering::Relaxed) {
                println!("Cancel order detected; ending thread.");
                return;
            }

            if let Ok(sensed) = sensed {
                data.lock().unwrap().push(sensed);
            }
        }
    });
    cloned_canceller
}

impl eframe::App for TemplateApp {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        if self.auto_repaint {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Import log").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            let name = path.file_name()
                                .expect("path should always point to a file")
                                .to_str().expect("filename should be valid unicode")
                                .to_owned();

                            match fs::read_to_string(path) {
                                Err(_) => self.set_short_status("Unable to read file".to_owned()),
                                Ok(text) => {
                                    self.data_source = DataSource::File { 
                                        name,
                                        data: text.split("--- S").last().unwrap()
                                            .lines()
                                            .map(|line| parse_log_line(line))
                                            .filter(|elem| elem.is_ok())
                                            .map(|elem| elem.unwrap())
                                            .collect() 
                                    };
                                },
                            }                            

                            ui.close_menu();
                        }
                    }

                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Connection", |ui| {
                    ui.menu_button("Port", |ui| {
                        if ui.radio(! matches!(self.data_source, DataSource::SerialPort {..}), "None").clicked() {
                            self.change_data_source(DataSource::None);
                        }
                    
                        if let Ok(ports) = serialport::available_ports() {
                            for port in ports.iter() {
                                let name = port.port_name.to_owned();
                                let port_chosen = {
                                    match &self.data_source {
                                        DataSource::SerialPort { port_name, .. } => *port_name == name,
                                        _ => false,
                                    }
                                };

                                if ui.radio(port_chosen, match &port.port_type {
                                    serialport::SerialPortType::UsbPort(info) => {
                                        format!(
                                            "{} ({}; {})", 
                                            name.to_owned(),
                                            info.product.clone().unwrap_or("Unknown".to_owned()),
                                            info.manufacturer.clone().unwrap_or("Unknown".to_owned())
                                        )
                                    },
                                    _ => name.to_owned()
                                }).clicked() {
                                    match serialport::new(name.to_owned(), 115200)
                                            .timeout(Duration::from_millis(1000))
                                            .open() {
                                        Err(e) => self.set_short_status(e.description),
                                        Ok(mut port) => {
                                            port.set_flow_control(serialport::FlowControl::Hardware).unwrap();
                                            let data: Arc<Mutex<Vec<SensedData>>> = Arc::new(Mutex::new(vec![]));

                                            self.change_data_source(DataSource::SerialPort {
                                                port_name: name.to_owned(),
                                                baud_rate: 115200,
                                                data: data.clone(),
                                                cancel_reader: spawn_data_reader_thread(BufReader::new(port), data)
                                            });
                                        },
                                    }
                                }
                            }
                        }
                    });
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.auto_repaint, "Repaint automatically");
                    egui::global_theme_preference_buttons(ui); 
                });

                ui.add_space(16.0);

                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center).with_cross_align(egui::Align::Center), |ui| {
                    ui.selectable_value(&mut self.current_tab, Tab::Data, "Data");
                    ui.selectable_value(&mut self.current_tab, Tab::Plot, "Plot");
                    ui.selectable_value(&mut self.current_tab, Tab::Map, "Map");
                });
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(match &self.data_source {
                    DataSource::File { name, data } 
                        => format!("Displaying {} lines from {}", data.len(), name),
                    DataSource::SerialPort { port_name, baud_rate, .. } 
                        => format!("Connected to serial {} at {} baud", port_name, baud_rate ),
                    DataSource::None 
                        => "No data.".to_owned(),
                });
    
                if let Some(status) = self.status_message.clone() {
                    if status.since.elapsed() > status.duration {
                        self.status_message = None;
                    }
                    
                    ui.separator();
                    ui.label("ℹ");
                    ui.label(status.text);
                    if ui.button("X").clicked() {
                        self.status_message = None;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {

            let mut data: Option<Vec<SensedData>> = None;
            self.run_with_data(|d| {
                if let Some(d) = d {
                    data = Some(d.to_vec());
                }
            });

            if let Some(data) = data {
                match self.current_tab {
                    Tab::Data => {
                        data_tab(ui, &mut self.data_state, &data);
                    },
                    Tab::Plot => {
                        plot_tab(ui, &mut self.plot_state, &data);
                    },
                    Tab::Map => {
                        map_tab(ui, &mut self.map_state, &data);
                    },
                }
            } else {
                ui.heading("No data available.");
                ui.label("Load a log file using File > Import log");
                ui.label("Or choose a device to connect to through Connection > Port > ...");
            }
            
        });   

    }
}

impl TemplateApp {
    fn run_with_data<F>(&self, closure: F) where F: FnOnce(Option<&Vec<SensedData>>) {
        match &self.data_source {
            DataSource::File { data, .. } => {
                closure(Some(&data));
            },
            DataSource::SerialPort { data, .. } => {
                let lock = data.lock().unwrap();
                
                closure(Some(&lock));
            },
            DataSource::None => {
                closure(None);
            },
        };
    }

    fn change_data_source(&mut self, new: DataSource) {
        if let DataSource::SerialPort { cancel_reader, .. } = &self.data_source {
            cancel_reader.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        self.data_source = new;
    }

    fn set_status(&mut self, text: String, duration: Duration) {
        self.status_message = Some(StatusMessage { 
            since: Instant::now(), duration, text 
        });
    }

    fn set_short_status(&mut self, text: String) {
        self.set_status(text, Duration::from_secs(6));
    }
}