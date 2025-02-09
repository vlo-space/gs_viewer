use egui::{Context, DragValue, Layout, Ui};
use walkers::{extras::{Place, Places, Style}, sources, HttpTiles, Map, MapMemory, Position};

use crate::data::SensedData;

pub struct MapTabState {
    map_memory: MapMemory,

    geo_view: bool,

    osm_tiles: HttpTiles,
    geo_tiles: HttpTiles,
}

impl MapTabState {
    pub fn new(egui_ctx: &Context) -> Self {
        MapTabState {
            map_memory: MapMemory::default(),
            geo_view: false,
            osm_tiles: HttpTiles::new(sources::OpenStreetMap, egui_ctx.clone()),
            geo_tiles: HttpTiles::new(sources::Geoportal, egui_ctx.clone())
        }
    }
}

pub fn map_tab<'a,'b>(
    ui: &mut Ui, 
    state: &mut MapTabState,
    data: &Vec<SensedData>
) {
    egui::SidePanel::left("map_side_panel").min_width(231.0).show_inside(ui, |ui| {
        ui.heading("Map settings");

        ui.checkbox(&mut state.geo_view, "Satellite view");

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
            Some(if state.geo_view {&mut state.geo_tiles} else {&mut state.osm_tiles}),
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