use std::f64::consts::PI;
use std::fs::File;
use std::io::prelude::*;
use nalgebra as na;

#[macro_export]
macro_rules! point3 {
    ($x:expr, $y:expr, $z:expr) => {
        na::Vector3::new($x as f64, $y as f64, $z as f64)
    };
}

pub struct Printer {
    pub position: na::Vector3<f64>,
    pub extruder: f64,
    gcode_file: File,
    travel_feedrate: i32,
    print_feedrate: i32,
    layer_height: f64,
    line_width: f64,
    filament_diameter: f64,
    flow_multiplier: f64,
    extrude_dist_per_travel: f64,
    file_cache: String
}

impl Printer {
    pub fn new(gcode_file: File, travel_feedrate: i32, print_feedrate: i32, layer_height: f64, line_width: f64, filament_diameter: f64, flow_multiplier: f64) -> Printer {
        let vol_per_moved_mm = layer_height * line_width; // mm^3/mm
        let vol_per_extruded_mm = (filament_diameter / 2.0).powf(2.0) * PI; // mm^3/mm
        let extrude_dist_per_travel = vol_per_moved_mm / vol_per_extruded_mm; // mm, somehow
        println!("Extruding {} per moved mm and {} per extruded mm so edpt is {}", vol_per_moved_mm, vol_per_extruded_mm, extrude_dist_per_travel);
        Printer {
            position: point3!(0, 0, 0),
            extruder: 0.0,
            gcode_file,
            travel_feedrate,
            print_feedrate,
            layer_height,
            line_width,
            filament_diameter,
            flow_multiplier,
            extrude_dist_per_travel,
            file_cache: String::new(),
        }
    }
    pub fn set_bed_temp(&mut self, temp: f64, wait: bool){
        if wait {
            self.file_cache += &format!("M190 S{}\n", temp);
        } else {
            self.file_cache += &format!("M140 S{}\n", temp);
        }
    }

    pub fn set_hotend_temp(&mut self, temp: f64, wait: bool){
        if wait {
            self.file_cache += &format!("M109 S{}\n", temp);
        } else {
            self.file_cache += &format!("M104 S{}\n", temp);
        }
    }

    pub fn set_fan(&mut self, speed: f64){
        self.file_cache += &format!("M106 S{}\n", (speed*255.0).clamp(0.0, 255.0).round());
    }

    pub fn home(&mut self){
        self.file_cache += &"G28\n";
        self.position = point3!(0, 0, 0);
    }

    pub fn absolute_extrusion(&mut self){
        self.file_cache += &"M82\n";
    }

    pub fn level_bed(&mut self){
        self.file_cache += &"G29\n";
    }

    pub fn set_extrusion(&mut self, extrusion: f64){
        self.file_cache += &format!("G92 E{}\n", extrusion);
        self.extruder = extrusion;
    }

    pub fn travel_to(&mut self, point: na::Vector3<f64>){
        self.file_cache += &format!("G0 X{} Y{} Z{} F{}\n", point.x, point.y, point.z, self.travel_feedrate);
        self.position = point;
    }

    pub fn set_travel_feedrate(&mut self, travel_feedrate: i32) {
        self.travel_feedrate = travel_feedrate;
    }

    pub fn set_print_feedrate(&mut self, print_feedrate: i32) {
        self.print_feedrate = print_feedrate;
    }

    pub fn extrude_to(&mut self, point: na::Vector3<f64>){
        let new_extruder_pos = self.get_extrude_dist(point) + self.extruder;
        self.file_cache += &format!("G1 X{} Y{} Z{} E{} F{}\n", point.x, point.y, point.z, new_extruder_pos, self.print_feedrate);
        self.position = point;
        self.extruder = new_extruder_pos;
    }

    pub fn get_extrude_dist(&self, point: na::Matrix<f64, na::Const<3>, na::Const<1>, na::ArrayStorage<f64, 3, 1>>) -> f64 {
        let new_extruder_pos: f64 = (point-self.position).magnitude() * self.extrude_dist_per_travel * self.flow_multiplier;
        new_extruder_pos
    }

    pub fn extrude_with_explicit_flow(&mut self, point: na::Vector3<f64>, flow_dist: f64){
        self.extruder += flow_dist;
        self.file_cache += &format!("G1 X{} Y{} Z{} E{} F{}\n", point.x, point.y, point.z, self.extruder, self.print_feedrate);
        self.position = point;
    }

    pub fn set_layer_height(&mut self, layer_height: f64) {
        self.layer_height = layer_height;
        let vol_per_moved_mm = self.layer_height * self.line_width; // mm^3/mm
        let vol_per_extruded_mm = (self.filament_diameter / 2.0).powf(2.0) * PI; // mm^3/mm
        self.extrude_dist_per_travel = vol_per_moved_mm / vol_per_extruded_mm; // mm, somehow
    }

    pub fn set_line_width(&mut self, line_width: f64) {
        self.line_width = line_width;
        let vol_per_moved_mm = self.layer_height * self.line_width; // mm^3/mm
        let vol_per_extruded_mm = (self.filament_diameter / 2.0).powf(2.0) * PI; // mm^3/mm
        self.extrude_dist_per_travel = vol_per_moved_mm / vol_per_extruded_mm; // mm, somehow
    }

    pub fn set_flow_multiplier(&mut self, flow_multiplier: f64) {
        self.flow_multiplier = flow_multiplier;
    }

    pub fn write_cache(&mut self){
        self.gcode_file.write(self.file_cache.as_bytes()).unwrap();
        self.file_cache = String::new();
    }

    pub fn move_extruder(&mut self, dist: f64){
        self.extruder += dist;
        self.file_cache += &format!("G1 E{} F300", self.extruder);
    }

    
}