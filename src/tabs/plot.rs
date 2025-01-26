use std::ops::{Add, Div, Mul};

use egui::{emath::Numeric, CollapsingHeader, Color32, Slider, Ui, WidgetText};
use egui_plot::{Legend, Line, Plot, PlotPoints};

use crate::data::SensedData;

struct LineSettings {
    visible: bool,
    offset: f64,
    scale: f64,
    min_absolute_value: f64,
    max_absolute_value: f64
}

impl Default for LineSettings {
    fn default() -> Self {
        Self { visible: true, offset: 0.0, scale: 1.0, min_absolute_value: 0.0, max_absolute_value: 0.0 }
    }
}

pub struct PlotTabState {
    acceleration: [LineSettings; 3],
    acceleration_sum: LineSettings,
    temperature: LineSettings,
    pressure: LineSettings,

    hide_nans: bool,

    filter_index_enabled: bool,
    filter_index_start: u32,
    filter_index_count: u32,

    filter_time_enabled: bool,
    filter_time_start: u64,
    filter_time_end: u64,
}

impl Default for PlotTabState {
    fn default() -> Self {
        Self { 
            acceleration: Default::default(), 
            acceleration_sum: Default::default(), 
            temperature: Default::default(), 
            pressure: Default::default(), 
            hide_nans: true,
            filter_index_enabled: false,
            filter_index_start: 0,
            filter_index_count: 200,
            filter_time_enabled: false,
            filter_time_start: 0,
            filter_time_end: 0,
        }
    }
}

fn duration_input(ui: &mut Ui, total: &mut u64) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 1.0;
        ui.add(egui::DragValue::from_get_set(|v: Option<f64>| {
            if let Some(v) = v {
                let h = v as u64;
                let m = *total / 60 % 60;
                let s = *total % 60;

                *total = h * 3600 + m * 60 + s;
            }
            (*total / 3600).to_f64()
        }).speed(0.1).range(0..=59).fixed_decimals(0));
        ui.label(":");
        ui.add(egui::DragValue::from_get_set(|v: Option<f64>| {
            if let Some(v) = v {
                let h = *total / 3600;
                let m = v as u64;
                let s = *total % 60;

                *total = h * 3600 + m * 60 + s;
            }
            (*total / 60 % 60).to_f64()
        }).speed(0.1).range(0..=59).fixed_decimals(0));
        ui.label(":");
        ui.add(egui::DragValue::from_get_set(|v: Option<f64>| {
            if let Some(v) = v {
                let h = *total / 3600;
                let m = *total / 60 % 60;
                let s = v as u64;

                *total = h * 3600 + m * 60 + s;
            }
            (*total % 60).to_f64()
        }).speed(0.1).range(0..=59).fixed_decimals(0));
    });
}

pub fn plot_tab(ui: &mut Ui, state: &mut PlotTabState, data: &Vec<SensedData>) {

    fn line_config(ui: &mut Ui, text: impl Into<WidgetText>, adjust: &mut LineSettings) {
        ui.checkbox(&mut adjust.visible, text);
        ui.add(egui::DragValue::new(&mut adjust.offset).speed(0.1));
        ui.add(egui::DragValue::new(&mut adjust.scale).speed(0.1));
        ui.add(egui::DragValue::new(&mut adjust.min_absolute_value).speed(0.1).range(0.0..=f64::MAX));
        ui.add(egui::DragValue::new(&mut adjust.max_absolute_value).speed(0.1).range(0.0..=f64::MAX));
    }

    egui::SidePanel::left("plot_side_panel").show_inside(ui, |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

        CollapsingHeader::new("Data lines").default_open(true).show(ui, |ui| {
            egui::Grid::new("some_unique_id").striped(true).show(ui, |ui| {
                ui.label("Line");
                ui.label("Offset");
                ui.label("Scale");
                ui.label("Min");
                ui.label("Max");
                ui.end_row();
    
                line_config(ui, "Acceleration X", &mut state.acceleration[0]);
                ui.end_row();
                
                line_config(ui, "Acceleration Y", &mut state.acceleration[1]);
                ui.end_row();
    
                line_config(ui, "Acceleration Z", &mut state.acceleration[2]);
                ui.end_row();
    
                line_config(ui, "Acceleration Sum", &mut state.acceleration_sum);
                ui.end_row();
    
                line_config(ui, "Temperature", &mut state.temperature);
                ui.end_row();
                
                line_config(ui, "Pressure", &mut state.pressure);
                ui.end_row();
            })
        });

        ui.separator();

        CollapsingHeader::new("Filter by index").default_open(true).show(ui, |ui| {
            ui.checkbox(&mut state.filter_index_enabled, "Enable index-based filtering");

            ui.add_space(4.0);

            ui.add_enabled_ui(state.filter_index_enabled, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Show");
                    ui.add(egui::DragValue::new(&mut state.filter_index_count).speed(1));
                    ui.label("records starting from index");
                });
    
                ui.spacing_mut().slider_width = 230.0;
                ui.add(Slider::new(&mut state.filter_index_start, 0..=data.len() as u32));
            });
        });

        ui.separator();

        CollapsingHeader::new("Filter by time").default_open(true).show(ui, |ui| {
            ui.checkbox(&mut state.filter_time_enabled, "Enable time-based filtering");

            ui.add_space(4.0);

            ui.add_enabled_ui(state.filter_time_enabled, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Start: ");
                    duration_input(ui, &mut state.filter_time_start);
                });
                ui.horizontal(|ui| {
                    ui.label("End: ");
                    duration_input(ui, &mut state.filter_time_end);
                });
                ui.horizontal(|ui| {
                    ui.label("Length: ");

                    if state.filter_time_end < state.filter_time_start {
                        state.filter_time_end = state.filter_time_start;
                    }

                    let mut length = state.filter_time_end - state.filter_time_start;
                    duration_input(ui, &mut length);
                    state.filter_time_end = state.filter_time_start + length;
                });
            });
        });

        ui.separator();

        ui.checkbox(&mut state.hide_nans, "Do not show missing data as gaps");

        ui.separator();
        ui.label("Double-click the plot to reset view");
    });

    let line: &dyn Fn(&str, Color32, &LineSettings, &dyn Fn(&SensedData) -> f64) -> Line = &|name, color, settings, processor| {
        Line::new(PlotPoints::new(
            data.iter()
                .filter_map(|s| {
                    let value: f64 = processor(s);

                    if (state.hide_nans && value.is_nan())
                        || (value.abs() < settings.min_absolute_value)
                        || (settings.max_absolute_value > 0.0 && value.abs() > settings.max_absolute_value)
                        || (state.filter_index_enabled && !(state.filter_index_start..=state.filter_index_start + state.filter_index_count).contains(&s.index) )
                        || (state.filter_time_enabled && !(state.filter_time_start..=state.filter_time_end).contains(&(s.uptime as u64 / 1000))) {
                        return None;
                    }
                    Some([s.uptime as f64, value * settings.scale + settings.offset])
                })
                .collect()
        ))
        .name(name)
        .color(color)
    };

    egui::CentralPanel::default().show_inside(ui, |ui| {
        Plot::new("plot")
            .legend(Legend::default())
            .auto_bounds([true, true].into())
            .show(ui, |plot_ui| {
                plot_ui.line(line("Acceleration X", Color32::from_rgb(239, 52, 80),  &state.acceleration[0], &|s| { s.acceleration[0] }));
                plot_ui.line(line("Acceleration Y", Color32::from_rgb(130, 202, 7),  &state.acceleration[1], &|s| { s.acceleration[1] }));
                plot_ui.line(line("Acceleration Z", Color32::from_rgb(43, 134, 231), &state.acceleration[2], &|s| { s.acceleration[2] }));

                plot_ui.line(line("Acceleration sum", Color32::from_rgb(195, 107, 176), &state.acceleration_sum, &|s| { 
                    (s.acceleration[0].powi(2) + s.acceleration[1].powi(2) + s.acceleration[2].powi(2)).sqrt()
                }));

                plot_ui.line(line("Temperature", Color32::from_rgb(43, 134, 231), &state.temperature, &|s| { s.temperature.to_f64() }));
                plot_ui.line(line("Pressure",    Color32::from_rgb(43, 134, 231), &state.pressure, &|s| { s.pressure.to_f64() }));
            });
    });
}