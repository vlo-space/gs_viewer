
use directories::ProjectDirs;
use egui::{color_picker::color_edit_button_rgba, CollapsingHeader, Color32, Context, DragValue, Frame, Layout, Rgba, RichText, Ui};
use walkers::{extras::{LabeledSymbol, LabeledSymbolStyle, Places}, sources, HttpOptions, HttpTiles, Map, MapMemory, Position};

use crate::data::SensedData;
use crate::util::map_trail::TrailPlugin;

pub struct MapTabState {
    map_memory: MapMemory,

    geo_view: bool,
    osm_tiles: HttpTiles,
    geo_tiles: HttpTiles,

    ground_station: GroundStationPosition,
    
    trail_color: Rgba,
    trail_length: usize,
}
#[derive(Default)]
struct GroundStationPosition {
    latitude: f64,
    longitude: f64,
}

impl MapTabState {
    pub fn new(egui_ctx: &Context) -> Self {
        let cache = ProjectDirs::from("eu", "vlospace", env!("CARGO_CRATE_NAME"))
            .map(|dirs| dirs.cache_dir().to_path_buf());

        MapTabState {
            map_memory: MapMemory::default(),
            geo_view: false,
            osm_tiles: HttpTiles::with_options(
                sources::OpenStreetMap,
                HttpOptions {
                    cache: cache.clone().map(|p| p.join("osm-tiles")),
                    ..Default::default()
                },
                egui_ctx.to_owned()
            ),
            geo_tiles: HttpTiles::with_options(
                sources::Geoportal,
                HttpOptions {
                    cache: cache.map(|p| p.join("geo-tiles")),
                    ..Default::default()
                },
                egui_ctx.to_owned()
            ),
            ground_station: GroundStationPosition::default(),
            trail_color: Color32::BLACK.into(),
            trail_length: 0
        }
    }
}

pub fn map_tab(
    ui: &mut Ui, 
    state: &mut MapTabState,
    data: &Vec<SensedData>
) {
    egui::SidePanel::left("map_side_panel").min_width(231.0).show_inside(ui, |ui| {

        CollapsingHeader::new("Map").default_open(true).show(ui, |ui| {
            ui.checkbox(&mut state.geo_view, "Satellite view");
        });

        ui.separator();

        CollapsingHeader::new("Camera").default_open(true).show(ui, |ui| {
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
                                state.map_memory.center_at(Position::new(v, position.y()));
                            }
                            position.x()
                        }).speed(speed));
                        ui.add(DragValue::from_get_set(|v: Option<f64>| {
                            if let Some(v) = v {
                                state.map_memory.center_at(Position::new(position.x(), v));
                            }
                            position.y()
                        }).speed(speed));
                    });
                });
            } else {
                ui.label("Following the probe");
            }
        });

        ui.separator();

        CollapsingHeader::new("Azimuth calculation").default_open(true).show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui|{
                ui.add_space(18.0);
                ui.heading("Azimuth");

                ui.label(RichText::new( if let Some(last) = data.last() {
                    let azimuth = calculate_azimuth(&[state.ground_station.latitude, state.ground_station.longitude], &last.gps_position );
                    format!("{:.2}", if azimuth < 0.0 {360.0 + azimuth} else {azimuth} )
                }
                else {
                    "-".to_string()
                }).size(40.0));
                ui.add_space(16.0);
            });
        
            ui.label("Ground station position");

            ui.add_space(2.0);

            ui.horizontal(|ui|{
                ui.label("Lat: ");
                ui.add(egui::DragValue::new(&mut state.ground_station.latitude).speed(0.1).range(-90.0..=90.0));
                ui.label("Lon: ");
                ui.add(egui::DragValue::new(&mut state.ground_station.longitude).speed(0.1).range(-180.0..=180.0));
            });

            ui.add_space(2.0);

            if ui.button("Set to probe position").clicked() {
                let position = data.last().map_or([0.0, 0.0], |d| d.gps_position);
                state.ground_station.latitude = position[0];
                state.ground_station.longitude = position[1];
            }
        });
    
        ui.separator();

        CollapsingHeader::new("Position trail").default_open(true).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Show last");
                ui.add(DragValue::new(&mut state.trail_length).range(0..=usize::MAX).speed(50.0));
                ui.label("positions");
            });
            ui.horizontal(|ui| {
                ui.label("Trail color: ");
                color_edit_button_rgba(ui, &mut state.trail_color, egui::color_picker::Alpha::Opaque);
            });
        });

        ui.separator();

        ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
            ui.label("Double-click to reset camera position");
        });
    });

    // egui::TopBottomPanel::bottom("map_bottom_panel").resizable(false).show_inside(ui, |ui| {
    //     ui.spacing_mut().slider_width = ui.available_width();
    //     ui.add(Slider::new(&mut state.index_slider_value, 0..=data.len()));
    // });

    let current_position = data.last().map_or(None, |s| Some(Position::new(s.gps_position[0], s.gps_position[1])));

    egui::CentralPanel::default().frame(Frame::NONE).show_inside(ui, |ui| {

        let map_response = ui.add(
            Map::new(
                Some(if state.geo_view {&mut state.geo_tiles} else {&mut state.osm_tiles}),
                &mut state.map_memory,
                current_position.unwrap_or(Position::new(0.0, 0.0))
            )
            .with_plugin(
                TrailPlugin {
                    positions: &mut data.iter().rev().take(state.trail_length).map(|data| {
                        Position::new(data.gps_position[0], data.gps_position[1])
                    }),
                    color: state.trail_color.into()
                }
            )
            .with_plugin({
                let mut points: Vec<LabeledSymbol> = vec![];

                if let Some(position) = current_position {
                    points.push(LabeledSymbol {
                        position,
                        label: "Latest location".to_owned(),
                        symbol: None,
                        style: LabeledSymbolStyle::default()
                    });
                }

                points.push(LabeledSymbol {
                    position: Position::new(state.ground_station.latitude, state.ground_station.longitude),
                    label: "Ground station".to_owned(),
                    symbol: None,
                    style: LabeledSymbolStyle::default()
                });

                Places::new(points)
            })
        );

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