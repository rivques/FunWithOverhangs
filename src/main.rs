pub mod easy_gcode_gen;
use easy_gcode_gen::Printer;
use nalgebra as na;
use std::{fs::File, f64::consts::PI};

fn main() {
    use std::time::Instant;
    let now = Instant::now();
    // constants
    let layer_height = 0.2; // mm
    let line_width = 0.4; // mm
    // let center_point = Point3d {x: 100.0, y: 100.0, z:layer_height};
    let hotend_temp = 195.0;// ℃
    let bed_temp = 55.0; // ℃
    let filament_dia = 1.75; // mm
    let travel_speed = 3000; // mm/min
    let print_speed = 2700; // mm/min
    let overhang_speed = 300; // mm/min
    let z_offset = 0.1; // mm

    // print specifics
    let disc_diameter = 30.0; // mm
    let axle_diameter = 10.0; // mm
    let file_name = "output\\double_mushroom.gcode";
    
    let file = File::create(file_name).unwrap();

    let mut printer = easy_gcode_gen::Printer::new(file, travel_speed, print_speed, layer_height, line_width, filament_dia,1.0);

    // let's start simple and print a line
    // setup
    printer.set_bed_temp(bed_temp, false);
    printer.set_hotend_temp(hotend_temp, false);
    printer.set_bed_temp(bed_temp, true);
    printer.set_hotend_temp(hotend_temp, true);
    printer.home();
    printer.level_bed();
    printer.absolute_extrusion();
    printer.set_extrusion(0.0);
    
    // purge line
    printer.set_flow_multiplier(2.0);
    printer.travel_to(point3!(30, 35, 0.4));
    printer.extrude_to(point3!(190, 35, 0.4));
    printer.set_flow_multiplier(1.0);
    printer.set_extrusion(0.0);
    printer.travel_to(printer.position + point3!(0, 0, 5));
    
    printer.travel_to(point3!(150, 150, 5));
    print_cylinder(&mut printer, disc_diameter, 5.0, point3!(150, 150, z_offset), line_width, layer_height, false);
    print_mushroom(&mut printer, axle_diameter, line_width, layer_height, overhang_speed, disc_diameter);
    print_mushroom(&mut printer, axle_diameter, line_width, layer_height, overhang_speed, disc_diameter);
    // retract, then raise up a bit
    printer.move_extruder(-5.0);
    printer.travel_to(point3!(printer.position.x, printer.position.y, 20.0 + printer.position.z));
    printer.travel_to(point3!(printer.position.x, 10, printer.position.z));
    printer.move_extruder(5.0);

    printer.write_cache();
    let elapsed = now.elapsed();
    println!("Generated in: {:.2?}", elapsed);
}

fn print_mushroom(printer: &mut Printer, axle_diameter: f64, line_width: f64, layer_height: f64, overhang_speed: i32, disc_diameter: f64) {
    let start_z = printer.position.z;
    print_cylinder(printer, axle_diameter, 5.0, point3!(150, 150, start_z), line_width, layer_height, false);
    let start_z = printer.position.z;
    printer.set_print_feedrate(overhang_speed);
    printer.set_fan(1.0);
    print_cylinder(printer, disc_diameter, layer_height, point3!(150, 150, start_z), 0.35, layer_height, false);
    print_cylinder(printer, disc_diameter, layer_height, point3!(150, 150, start_z+layer_height), 0.4, layer_height, false);
    printer.set_print_feedrate(2000);
    let start_z = printer.position.z;
    print_cylinder(printer, disc_diameter, 5.0-layer_height*2.0, point3!(150, 150, start_z), 0.4, layer_height, true);
}

fn print_cylinder(printer: &mut Printer, diameter: f64, height: f64, starting_location: na::Vector3<f64>, spacing: f64, layer_height: f64, distance_decay: bool){
    // let's spiral out, a few times on top of each other
    for layer in 1..=((height/layer_height).floor() as i32) {
        printer.travel_to(starting_location + point3!(0, 0, layer as f64*layer_height));
        for theta_deg in (0..(360*(diameter/2.0/spacing).floor() as i32)).step_by(5) {
            let theta = theta_deg as f64 * PI / 180.0;
            let r = (spacing / (2.0*PI)) * theta;
            let point = point3!(r*theta.cos() + starting_location.x, r*theta.sin() + starting_location.y, printer.position.z);
            if distance_decay{
                printer.extrude_with_explicit_flow(point, circle_decay_flow(Printer::get_extrude_dist(&printer, point), r, diameter));
            } else {
                printer.extrude_to(point)
            }
        }
        printer.set_flow_multiplier(1.0);
    }
}

fn circle_decay_flow(in_flow: f64, r: f64, diameter: f64) -> f64 {
    let factor = (-2.0/diameter*r).powf(2.0) + 1.0;
    in_flow * factor
}

// fn map(x: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
//     return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
// }