use egui::{Color32, Stroke, Ui};
use walkers::{extras::{Place, Places, Style}, Map, MapMemory, Position, Tiles};

use crate::data::SensedData;

pub fn map_tab<'a,'b>(
    ui: &mut Ui, 
    _data: &Vec<SensedData>, 
    tiles: Option<&'b mut dyn Tiles>, 
    memory: &'a mut MapMemory
) {
    egui::SidePanel::left("map_side_panel").show_inside(ui, |ui| {
        ui.label("Map settings.");
    });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        ui.add(Map::new(
            tiles,
            memory,
            Position::from_lon_lat(19.51713, 50.34858)
        ).with_plugin(
            {
                let mut points: Vec<Place> = vec![];
    
                for i in 0..=16 {
                    let alpha = ((32.0 - i as f64) / 32.0 * 20.0) as u8 + 1;
                    let color = ((32.0 - i as f64) / 32.0 * 128.0) as u8;
                    points.push(Place {
                        position: Position::from_lon_lat(19.51720 - (i as f64) * 0.0004, 50.34859 - (i as f64) * 0.0004),
                        label: format!("{i}").to_owned(),
                        symbol: ' ',
                        style: Style { 
                            label_font: Style::default().label_font, 
                            label_color: Color32::TRANSPARENT, 
                            label_background: Color32::TRANSPARENT, 
                            symbol_font: Style::default().symbol_font, 
                            symbol_color: Color32::TRANSPARENT,
                            symbol_background: Color32::from_rgba_unmultiplied(color, color, color, alpha),
                            symbol_stroke: Stroke { width: 2.0, color: Color32::from_rgba_unmultiplied(50, 50, 50, 25) }
                        }
                    });
                }
    
                points.push(Place {
                    position: Position::from_lon_lat(19.51713, 50.34858),
                    label: "Latest location".to_owned(),
                    symbol: ' ',
                    style: Style::default()
                });
    
                Places::new(points)
            }
        ))
    }); 
}