use egui::{Color32, Pos2, Stroke, Ui};
use walkers::{Plugin, Position};

pub struct TrailPlugin<'a> {
    pub positions: &'a mut dyn Iterator<Item = Position>,
    pub color: Color32
}

impl Plugin for TrailPlugin<'_> {
     fn run(self: Box<Self>, ui: &mut Ui, _response: &egui::Response, projector: &walkers::Projector, _memory: &walkers::MapMemory) {
        let mut prev_position: Option<Pos2> = None;

        for position in self.positions {
            let projected = projector.project(position).to_pos2();

            if let Some(prev_position) = prev_position {
                ui.painter().line_segment(
                    [prev_position, projected],
                    Stroke::new(6.0, self.color)
                );
            }

            prev_position = Some(projected);
        }
    }
}