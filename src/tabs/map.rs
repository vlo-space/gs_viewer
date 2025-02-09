use egui::{DragValue, Layout, Ui};
use walkers::{extras::{Place, Places, Style}, Map, MapMemory, Position, Tiles};

use crate::data::SensedData;

#[derive(Default)]
pub struct MapTabState {
    pub map_memory: MapMemory
}

pub fn map_tab<'a,'b>(
    ui: &mut Ui, 
    state: &mut MapTabState,
    data: &Vec<SensedData>, 
    tiles: Option<&'b mut dyn Tiles>, 
) {
    egui::SidePanel::left("map_side_panel").min_width(231.0).show_inside(ui, |ui| {
        ui.heading("Map settings");

        ui.label("Camera position");
        let speed = 0.01 / state.map_memory.zoom();
        if let Some(position) = state.map_memory.detached() {
            egui::Grid::new("map_position_grid").show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Reset").clicked() {
                        state.map_memory.follow_my_position();
                    }
                    ui.add(DragValue::from_get_set(|v: Option<f64>| {
                        if let Some(v) = v {
                            state.map_memory.center_at(Position::from_lat_lon(v, position.lon()));
                        }
                        position.lat()
                    }).speed(speed));
                    ui.add(DragValue::from_get_set(|v: Option<f64>| {
                        if let Some(v) = v {
                            state.map_memory.center_at(Position::from_lat_lon(position.lat(), v));
                        }
                        position.lon()
                    }).speed(speed));
                });
            });
        } else {
            ui.label("Following the probe");
        }

        ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
            ui.label("Double-click to reset camera position");
        });
    });

    // egui::TopBottomPanel::bottom("map_bottom_panel").resizable(false).show_inside(ui, |ui| {
    //     ui.spacing_mut().slider_width = ui.available_width();
    //     ui.add(Slider::new(&mut state.index_slider_value, 0..=data.len()));
    // });

    let current_position = data.last().map_or(None, |s| Some(Position::from_lat_lon(s.gps_position[0], s.gps_position[1])));

    egui::CentralPanel::default().show_inside(ui, |ui| {
        let map_response = ui.add(Map::new(
            tiles,
            &mut state.map_memory,
            current_position.unwrap_or(Position::from_lat_lon(0.0, 0.0))
        ).with_plugin({
            let mut points: Vec<Place> = vec![];

            if let Some(position) = current_position {
                points.push(Place {
                    position,
                    label: "Latest location".to_owned(),
                    symbol: ' ',
                    style: Style::default()
                });
            }

            Places::new(points)
        }));

        if map_response.double_clicked() {
            state.map_memory.follow_my_position();
        }
    }); 
}