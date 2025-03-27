use plotters::prelude::LogScalable;
use tcp_plotter::tcp_receiver::Cordinate;
use serde_json::Result;
use std::fs::File;
use std::io::Write;

#[test]
fn generate_cords() -> Result<()> {
    let mut cordinates: Vec<Cordinate> = Vec::new();
    
    for i in 1..4096 {
        cordinates.push(Cordinate { x: i.as_f64()/4096f64, y: i.as_f64()/4096f64 });
    }

    let json = serde_json::to_string(&cordinates)?;
    println!("Output:\n {json}");
    let mut json_file = File::create("tests/cords.txt").expect("Failed to create file");
    json_file.write_all([json, "\n".to_string()].concat().as_bytes()).expect("Failed to write to file");
    
    Ok(())
}