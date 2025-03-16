use egui::{Context, DragValue, Layout, RichText, Ui};
use walkers::{extras::{Place, Places, Style}, sources, HttpTiles, Map, MapMemory, Position};

use crate::data::SensedData;

pub struct MapTabState {
    map_memory: MapMemory,

    geo_view: bool,

    osm_tiles: HttpTiles,
    geo_tiles: HttpTiles,

    ground_station: GroundStationPosition
}
#[derive(Default)]
struct GroundStationPosition {
    latitude: f64,
    longitude: f64,
    altitude: f64,
 }

impl MapTabState {
    pub fn new(egui_ctx: &Context) -> Self {
        MapTabState {
            map_memory: MapMemory::default(),
            geo_view: false,
            osm_tiles: HttpTiles::new(sources::OpenStreetMap, egui_ctx.clone()),
            geo_tiles: HttpTiles::new(sources::Geoportal, egui_ctx.clone()),
            ground_station: GroundStationPosition::default()
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

        ui.add_space(32.0);
        
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui|{
            ui.add_space(16.0);
            ui.heading("Azimuth");

            ui.label(RichText::new( if let Some(last) = data.last() {
                format!("{}", calculate_azimuth(&[state.ground_station.latitude, state.ground_station.longitude], &last.gps_position ))
            }
            else {
                "-".to_string()
            }).size(40.0));
            ui.add_space(16.0);
        });
        
        ui.label("Ground station position");
       
        ui.horizontal(|ui|{
            ui.label("Lat: ");
            ui.add(egui::DragValue::new(&mut state.ground_station.latitude).speed(0.1).range(-90.0..=90.0));
            ui.label("Lon: ");
            ui.add(egui::DragValue::new(&mut state.ground_station.longitude).speed(0.1).range(-180.0..=180.0));
            ui.label("Alt: ");
            ui.add(egui::DragValue::new(&mut state.ground_station.altitude).speed(0.1).range(0.0..=f64::MAX));
        });
    
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

pub fn calculate_azimuth(gs_cords: &[f64; 2], probe_cords: &[f64; 2]) -> f64{
    let lon_diff = (probe_cords[1] - gs_cords[1]).to_radians();
    let intial_point: [f64;2] = [gs_cords[0].to_radians(), gs_cords[1].to_radians()];
    let end_point: [f64;2] = [probe_cords[0].to_radians(), probe_cords[1].to_radians()];

    (lon_diff.sin() * end_point[0].cos()).atan2(intial_point[0].cos() * end_point[0].sin() - intial_point[0].sin() * end_point[0].cos() * lon_diff.cos()).to_degrees()
}