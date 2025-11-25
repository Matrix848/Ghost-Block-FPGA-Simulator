use anyhow::{Context, Result};
use rfd::FileDialog;
use simulator_core::FPGA;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct File {
    path: PathBuf,
}

impl File {
    pub(crate) fn save_fpga(&self, fpga: &FPGA) -> Result<()> {
        let mut file = fs::File::create(self.path.clone())?;
        let encoded = postcard::to_allocvec(fpga)?;
        file.write_all(&encoded)?;

        Ok(())
    }

    pub(crate) fn load_fpga(&mut self, fpga: &mut FPGA) -> Result<()> {
        let data = fs::read(self.path.clone())?;
        *fpga = postcard::from_bytes(&data)?;

        Ok(())
    }

    pub fn save(&self, fpga: &mut FPGA) -> Result<()> {
        self.save_fpga(fpga)
    }

    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }
    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }

    pub fn open_file_dialog(&mut self, fpga: &mut FPGA) -> Result<()> {
        self.path = FileDialog::new()
            .add_filter("FPGA Configuration File", &["fpga", "bit"])
            .add_filter("All Files", &["*"])
            .set_title("Choose a FPGA configuration file")
            .pick_file()
            .context("No path selected")?;

        self.load_fpga(fpga)?;

        Ok(())
    }

    pub fn save_as_dialog(&mut self, fpga: &mut FPGA) -> Result<()> {
        self.path = FileDialog::new()
            .add_filter("FPGA Configuration File", &["fpga", "bit"])
            .add_filter("All Files", &["*"])
            .set_title("Choose a FPGA configuration file")
            .save_file()
            .context("Invalid path")?;

        self.save_fpga(fpga)?;

        Ok(())
    }
}
