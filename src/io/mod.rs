use anyhow::{Context, Result};
use rfd::FileDialog;
use simulator_core::FPGA;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct File {
    path: Option<PathBuf>,
    pub(crate) fpga: FPGA,
}

impl File {
    pub(crate) fn save_fpga(&self) -> Result<()> {
        let mut file = fs::File::create(self.path.as_ref().context("No Path specified")?)?;
        let encoded = postcard::to_allocvec(&self.fpga)?;
        file.write_all(&encoded)?;

        Ok(())
    }

    pub(crate) fn load_fpga(&mut self) -> Result<()> {
        let data = fs::read(self.path.as_ref().context("No Path specified")?)?;
        self.fpga = postcard::from_bytes(&data)?;

        Ok(())
    }

    pub fn open_file_dialog(&mut self) -> Result<()> {
        self.path = FileDialog::new()
            .add_filter("FPGA Configuration File", &["fpga, bit"])
            .add_filter("All Files", &["*"])
            .set_title("Choose a FPGA configuration file")
            .pick_file();

        self.load_fpga()?;

        Ok(())
    }

    pub fn save_as(&mut self) -> Result<()> {
        self.path = FileDialog::new()
            .add_filter("FPGA Configuration File", &["fpga, bit"])
            .add_filter("All Files", &["*"])
            .set_title("Choose a FPGA configuration file")
            .save_file();

        self.save_fpga()?;

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        self.save_fpga()
    }

    pub fn get_path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&simulator_core::cell::Cell> {
        self.fpga.get_cell(row, col)
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        self.path = path;
    }
}
