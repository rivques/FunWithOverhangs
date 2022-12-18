pub mod easy_gcode_gen;
use easy_gcode_gen::Printer;
use nalgebra as na;
use std::{fs::File, f64::consts::PI};

#[allow(unused_variables)]
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
    let travel_speed = 2000; // mm/min
    let print_speed = 1000; // mm/min
    let overhang_speed = 300; // mm/min
    let overhang_flow = 1.2;

    // print specifics
    let disc_diameter = 30.0; // mm
    let axle_diameter = 10.0; // mm
    let file_name = "output\\spiral.gcode";
    
    let file = File::create(file_name).unwrap();

    let mut printer = easy_gcode_gen::Printer::new(file, travel_speed, print_speed, layer_height, line_width, filament_dia,1.0);

    // let's start simple and print a line
    // setup
    printer.set_bed_temp(bed_temp, false);
    printer.set_hotend_temp(hotend_temp, false);
    printer.set_bed_temp(bed_temp, true);
    printer.set_hotend_temp(hotend_temp, true);
    printer.home();
    //printer.level_bed();
    printer.absolute_extrusion();
    printer.set_extrusion(0.0);
    
    // purge line
    printer.set_flow_multiplier(2.0);
    printer.travel_to(point3!(30, 35, 0.3));
    printer.extrude_to(point3!(190, 35, 0.25));
    printer.set_flow_multiplier(1.0);
    printer.set_extrusion(0.0);
    printer.travel_to(printer.position + point3!(0, 0, 5));
    
    printer.travel_to(point3!(150, 150, 5));
    print_cylinder(&mut printer, disc_diameter, 5.0, point3!(150, 150, 0.1), line_width, layer_height);
    let start_z = printer.position.z;
    print_cylinder(&mut printer, axle_diameter, 5.0, point3!(150, 150, start_z), line_width, layer_height);
    let start_z = printer.position.z;
    printer.set_print_feedrate(300);
    printer.set_flow_multiplier(1.2);
    print_cylinder(&mut printer, disc_diameter, 5.0, point3!(150, 150, start_z), line_width, layer_height);
    // raise up a bit
    printer.travel_to(point3!(printer.position.x, printer.position.y, 20.0 + printer.position.z));
    printer.travel_to(point3!(printer.position.x, 10, printer.position.z));

    printer.write_cache();
    let elapsed = now.elapsed();
    println!("Generated in: {:.2?}", elapsed);
}

fn print_cylinder(printer: &mut Printer, diameter: f64, height: f64, starting_location: na::Vector3<f64>, spacing: f64, layer_height: f64){
    // let's spiral out, a few times on top of each other
    for layer in 1..((height/layer_height).floor() as i32) {
        printer.travel_to(starting_location + point3!(0, 0, layer as f64*layer_height));
        for theta_deg in (0..(360*(diameter/2.0/spacing).floor() as i32)).step_by(5) {
            let theta = theta_deg as f64 * PI / 180.0;
            let r = (spacing/ (2.0*PI) ) * theta as f64;
            printer.extrude_to(point3!(r*theta.cos() + starting_location.x, r*theta.sin() + starting_location.y, printer.position.z));
        }
        printer.set_flow_multiplier(1.0);
    }
}