use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use zip::result::ZipResult;
use zip::ZipWriter;
use zip::write::FileOptions;

use super::Compiler;

impl Compiler {
    fn save(&self, path:&Path) -> ZipResult<()> {
        let mut zw = ZipWriter::new(
            File::create(format!("{}.zip", self.namespace))?
        );
        let opts:FileOptions<'_, ()> = FileOptions::default();

        for mcf in &self.compiled {
            let mut path = path.join(mcf.path.iter().collect::<PathBuf>());
            path.push(format!("{}.mcfunction", mcf.name));
            zw.start_file(path.to_string_lossy().as_ref(), opts)?;
            zw.write(mcf.inside.as_bytes())?;
        }

        zw.flush()?;
        zw.finish()?;

        Ok(())
    }
}