
use egui::{Ui, Vec2};
use egui_extras::{Column, TableBuilder};

use crate::data::SensedData;

pub struct DataTabState {
    pub stick_to_bottom: bool
}

pub fn data_tab(ui: &mut Ui, state: &mut DataTabState, data: &Vec<SensedData>) {
    let text_height = egui::TextStyle::Body
        .resolve(ui.style())
        .size
        .max(ui.spacing().interact_size.y);

    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

    egui::SidePanel::left("data_side_panel").show_inside(ui, |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

        ui.checkbox(&mut state.stick_to_bottom, "Stick to bottom");

        ui.separator();

        ui.heading("Stats");
        if data.is_empty() {
            ui.label("No messages recieved");
        } else {
            let recieved = data.len() as u32;
            let total = data.last().unwrap().index - data.first().unwrap().index + 1;

            ui.label(format!("Recieved: {}", recieved));
            ui.label(format!("Total: {}", total));
            ui.label(format!("Lost: {}, {}%", total - recieved, (total - recieved) as f32 / (total as f32)));

            ui.allocate_space(Vec2 { x: 0.0, y: 10.0 });

            let mut max_time = 0;
            let mut min_time = 0;
            let mut sum = 0;
            let mut count = 0;

            let mut prev_uptime = 0;
            for row in data {
                if prev_uptime != 0 {
                    if let Some(delta) = row.uptime.checked_sub(prev_uptime) {
                        if delta > max_time {
                            max_time = delta;
                        }

                        if delta < min_time {
                            min_time = delta;
                        }

                        sum += delta;
                        count += 1;
                    }
                }

                prev_uptime = row.uptime;
            }

            ui.label(format!("Avg time between messages: {}", sum as f32 / count as f32));
            ui.label(format!("Max time between messages: {}", max_time));
            ui.label(format!("Min time between messages: {}", min_time));
            ui.weak("Records with a negative time delta are ignored.");
        }
        
    });
    
    egui::CentralPanel::default().show_inside(ui, |ui| {
        TableBuilder::new(ui)
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .stick_to_bottom(state.stick_to_bottom)
            .header(20.0, |mut header| {
                header.col(|ui| {ui.label("#");});
                header.col(|ui| {ui.label("Uptime");});
                header.col(|ui| {ui.label("GPS Time");});
                header.col(|ui| {ui.label("Temperature");});
                header.col(|ui| {ui.label("Pressure");});
                header.col(|ui| {ui.label("GPS Location");});
                header.col(|ui| {ui.label("GPS Altitude");});
                header.col(|ui| {ui.label("Accel X");});
                header.col(|ui| {ui.label("Accel Y");});
                header.col(|ui| {ui.label("Accel Z");});
            })
            .body(|body| {
                body.rows(text_height, data.len(), |mut row| {

                    let row_index = row.index();

                    let data_row = &data[row_index];

                    row.col(|ui| {
                        ui.weak(row_index.to_string());
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", data_row.uptime));
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", data_row.gps_time));
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", data_row.temperature));
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", data_row.pressure));
                    });
                    row.col(|ui| {
                        if data_row.gps_position[0].is_nan() || data_row.gps_position[1].is_nan() {
                            ui.weak("?");
                        } else {
                            ui.label(format!("{} {}", data_row.gps_position[0], data_row.gps_position[1]));
                        }
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", data_row.gps_altitude));
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", data_row.acceleration[0]));
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", data_row.acceleration[1]));
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", data_row.acceleration[2]));
                    });
                });
            });
    });
    
}