use crate::fpga::Grid;
use std::fs;
use std::io::Write;

fn save_grid(grid: &Grid, path: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    let encoded = postcard::to_allocvec(grid).expect("Unable to serialize grid");
    file.write(&encoded).expect(&format!("Unable to write to {path}"));
    Ok(())
}

fn load_grid(grid: &Grid, path: &str) -> std::io::Result<Grid> {
    let file = fs::read(path)?;
    let grid: Grid = postcard::from_bytes(&file).expect("Unable to deserialize grid");
    Ok(grid)
}