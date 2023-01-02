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
    let file_name = "output\\small_roller.gcode";
    
    let file = File::create(file_name).unwrap();

    let mut printer = easy_gcode_gen::Printer::new(file, travel_speed, print_speed, layer_height, line_width, filament_dia,1.0);

    // let's start simple and print a line
    // setup
    printer.comment("Double mushroom");
    printer.set_bed_temp(bed_temp, false);
    printer.set_hotend_temp(hotend_temp, false);
    printer.set_bed_temp(bed_temp, true);
    printer.set_hotend_temp(hotend_temp, true);
    printer.home();
    printer.level_bed();
    printer.absolute_extrusion();
    printer.set_extrusion(0.0);
    
    // purge line
    printer.comment("Purge line");
    printer.set_flow_multiplier(2.0);
    printer.travel_to(point3!(30, 35, 0.4));
    printer.extrude_to(point3!(190, 35, 0.4));
    printer.set_flow_multiplier(1.0);
    printer.set_extrusion(0.0);
    printer.travel_to(printer.position + point3!(0, 0, 5));
    
    printer.comment("begin cylinder");
    printer.travel_to(point3!(150, 150, 5));
    print_cylinder(&mut printer, disc_diameter, 10.0, point3!(150, 150, z_offset), line_width, layer_height, false);
    print_mushroom(&mut printer, axle_diameter, line_width, layer_height, overhang_speed, disc_diameter, print_speed);
    print_mushroom(&mut printer, axle_diameter, line_width, layer_height, overhang_speed, disc_diameter, print_speed);
    print_mushroom(&mut printer, axle_diameter, line_width, layer_height, overhang_speed, disc_diameter, print_speed);
    // retract, then raise up a bit
    printer.comment("retract");
    printer.move_extruder(-5.0);
    printer.travel_to(point3!(printer.position.x, printer.position.y, 20.0 + printer.position.z));
    printer.travel_to(point3!(printer.position.x, 10, printer.position.z));
    printer.move_extruder(5.0);
    printer.set_bed_temp(0.0, false);
    printer.set_hotend_temp(0.0, false);

    let print_time = printer.get_time_spent();
    let seconds = print_time.as_secs() % 60;
    let minutes = (print_time.as_secs() / 60) % 60;
    let hours = (print_time.as_secs() / 60) / 60; 
    printer.comment(&format!("Print took {}:{}:{} and used {:.3}m of filament", hours, minutes, seconds, (printer.get_dist_extruded()/1000.0)));
    println!("Print took {}:{}:{} and used {:.3}m of filament", hours, minutes, seconds, (printer.get_dist_extruded()/1000.0));

    printer.write_cache();
    let elapsed = now.elapsed();
    println!("Generated in: {:.2?}", elapsed);
}

fn print_mushroom(printer: &mut Printer, axle_diameter: f64, line_width: f64, layer_height: f64, overhang_speed: i32, disc_diameter: f64, print_speed: i32) {
    let start_z = printer.position.z;
    printer.comment("Printing mushroom");
    print_cylinder(printer, axle_diameter, 10.0, point3!(150, 150, start_z), line_width, layer_height, false);
    let start_z = printer.position.z;
    printer.comment("Printing overhang");
    printer.set_print_feedrate(overhang_speed);
    printer.set_fan(1.0);
    print_cylinder(printer, disc_diameter, layer_height, point3!(150, 150, start_z), 0.35, layer_height, false);
    print_cylinder(printer, disc_diameter, layer_height, point3!(150, 150, start_z+layer_height), 0.4, layer_height, false);
    printer.set_print_feedrate(print_speed);
    printer.comment("printing with decay");
    let start_z = printer.position.z;
    print_cylinder(printer, disc_diameter, 3.0-layer_height*2.0, point3!(150, 150, start_z), 0.4, layer_height, true);
    printer.comment("end decay");
    let start_z = printer.position.z;
    print_cylinder(printer, disc_diameter, 10.0-(3.0-layer_height*2.0), point3!(150, 150, start_z), line_width, layer_height, false);
}

fn print_cylinder(printer: &mut Printer, diameter: f64, height: f64, starting_location: na::Vector3<f64>, spacing: f64, layer_height: f64, distance_decay: bool){
    // let's spiral out, a few times on top of each other
    for layer in 1..=((height/layer_height).floor() as i32) {
        //println!("Now on layer {}/{}, decaying? {}", layer, ((height/layer_height).floor() as i32), distance_decay);
        printer.move_extruder(-3.0);
        printer.travel_to(starting_location + point3!(0, 0, layer as f64*layer_height));
        printer.move_extruder(3.0);

        for theta_deg in (0..(360*(diameter/2.0/spacing).floor() as i32)).step_by(5) {
            let theta = theta_deg as f64 * PI / 180.0;
            let r = (spacing / (2.0*PI)) * theta;
            let point = point3!(r*theta.cos() + starting_location.x, r*theta.sin() + starting_location.y, printer.position.z);
            if distance_decay{
                let print_factor = circle_decay_flow_factor(r, diameter);
                let in_flow = Printer::get_extrude_dist(&printer, point);
                printer.extrude_with_explicit_flow(point, print_factor*in_flow);
                //printer.comment(&format!("Decayed flow: r is {}, dia {}, factor {}, old flow {}", r, diameter, print_factor, in_flow));
            } else {
                printer.extrude_to(point)
            }
        }
        printer.set_flow_multiplier(1.0);
    }
}

fn circle_decay_flow_factor(r: f64, diameter: f64) -> f64 {
    -(2.0/(1.05*diameter)*r).powf(2.0) + 1.0
}

// fn map(x: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
//     return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
// }