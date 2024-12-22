use egui::{emath::Numeric, Color32, Ui};
use egui_plot::{Legend, Line, Plot, PlotPoints};

use crate::data::SensedData;

pub struct PlotTabState {
    pub accel_sum_visible: bool,
    pub accel_visible: [bool; 3],
    pub temperature_visible: bool,
    pub pressure_visible: bool,

    pub aspect: f32,

    pub legend: Legend,

    pub acceleration_cutoff: [f64; 3],
    pub acceleration_offset: [f64; 3],
}

pub fn plot_tab(ui: &mut Ui, state: &mut PlotTabState, data: &Vec<SensedData>) {
    egui::SidePanel::left("plot_side_panel").show_inside(ui, |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

        ui.label("Visible items");
        ui.checkbox(&mut state.accel_sum_visible, "Acceleration Sum");
        ui.checkbox(&mut state.accel_visible[0], "Acceleration X");
        ui.checkbox(&mut state.accel_visible[1], "Acceleration Y");
        ui.checkbox(&mut state.accel_visible[2], "Acceleration Z");
        ui.checkbox(&mut state.temperature_visible, "Temperature");
        ui.checkbox(&mut state.pressure_visible, "Pressure");

        ui.separator();

        ui.label("Data filtering");
        ui.label("Acceleration Cutoff");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut state.acceleration_cutoff[0]).speed(0.01));
            ui.add(egui::DragValue::new(&mut state.acceleration_cutoff[1]).speed(0.01));
            ui.add(egui::DragValue::new(&mut state.acceleration_cutoff[2]).speed(0.01));
        });
        ui.label("Acceleration Offset");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut state.acceleration_offset[0]).speed(0.01));
            ui.add(egui::DragValue::new(&mut state.acceleration_offset[1]).speed(0.01));
            ui.add(egui::DragValue::new(&mut state.acceleration_offset[2]).speed(0.01));
        });
    });

    let get_acceleration = |data: &SensedData, coord: usize| {
        data.acceleration[coord] + state.acceleration_offset[coord]
    };

    let accel_sum_line = Line::new(PlotPoints::new(data.iter()
        .map(|d| [
            d.uptime.to_f64(),
            (get_acceleration(&d, 0).powi(2) + get_acceleration(&d, 1).powi(2) + get_acceleration(&d, 2).powi(2)).sqrt()
        ])
        .collect()))
        .name("Acceleration sum")
        .color(Color32::from_rgb(195, 107, 176));

    let line_for_acceleration = |coord: usize| {
        Line::new(PlotPoints::new(
            data.iter().map(|d| {
                [
                    d.uptime.to_f64(), 
                    get_acceleration(&d, coord)
                ]
            }).collect()
        ))
    };

    let accel_x = line_for_acceleration(0)
        .name("Acceleration X")
        .color(Color32::from_rgb(239, 52, 80));

    let accel_y = line_for_acceleration(1)
        .name("Acceleration Y")
        .color(Color32::from_rgb(130, 202, 7));

    let accel_z = line_for_acceleration(2)
        .name("Acceleration Z")
        .color(Color32::from_rgb(43, 134, 231));

    let temperature = Line::new(PlotPoints::new(
        data.iter().map(|d| [d.uptime.to_f64(), d.temperature.to_f64()]).collect()
    )).name("Temperature");

    let pressure = Line::new(PlotPoints::new(
        data.iter().map(|d| [d.uptime.to_f64(), d.pressure.to_f64()]).collect()
    )).name("Pressure");

    egui::CentralPanel::default().show_inside(ui, |ui| {
        Plot::new("plot")
            .legend(Legend::default())
            .auto_bounds([true, true].into())
            .show(ui, |plot_ui| {
                if state.accel_sum_visible   { plot_ui.line(accel_sum_line);}
                if state.accel_visible[0]    { plot_ui.line(accel_x); }
                if state.accel_visible[1]    { plot_ui.line(accel_y); }
                if state.accel_visible[2]    { plot_ui.line(accel_z); }
                if state.temperature_visible { plot_ui.line(temperature); }
                if state.pressure_visible    { plot_ui.line(pressure); }
            });
    });
}