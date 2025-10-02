use simulator_core::FPGA;
use std::fs;
use std::io::Write;

pub(crate) fn save_fpga(fpga: &FPGA, path: &str) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;
    let encoded = postcard::to_allocvec(fpga).expect("Unable to serialize grid");
    file.write(&encoded)
        .expect(&format!("Unable to write to {path}"));
    Ok(())
}

pub(crate) fn load_fpga(path: &str) -> std::io::Result<FPGA> {
    let file = fs::read(path)?;
    let grid: FPGA = postcard::from_bytes(&file).expect("Unable to deserialize grid");
    Ok(grid)
}
