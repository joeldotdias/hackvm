use std::{
    env, fs,
    io::{self, Write},
    path::Path,
};

use hackvm::{parse, VMTranslator};

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    assert!(args.len() >= 2, "Usage: hackvm <filename>.vm");
    let inpath = Path::new(&args[1]);

    if inpath.is_file() {
        assert!(
            inpath.extension().and_then(|ext| ext.to_str()) == Some("vm"),
            "Expected .vm file"
        );

        let content = fs::read_to_string(inpath)?;
        let mut translator = VMTranslator::new(inpath)?;
        translator.write_prelude()?;

        write_file_asm(&mut translator, content)?;
    } else if inpath.is_dir() {
        let infiles: Vec<_> = fs::read_dir(inpath)?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("vm") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert!(
            !infiles.is_empty(),
            "No .vm files found in the specified directory"
        );

        let mut translator = VMTranslator::new(inpath)?;
        translator.write_prelude()?;

        for infile in infiles {
            let content = fs::read_to_string(&infile)?;
            translator.update_filestem(&infile);

            write_file_asm(&mut translator, content)?;
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Expected a file or directory"),
        ));
    }

    Ok(())
}

fn write_file_asm<W: Write>(translator: &mut VMTranslator<W>, content: String) -> io::Result<()> {
    for (n, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        match parse(line) {
            Ok(command) => {
                translator.write_asm(command)?;
            }

            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Error at line {}: {}", n + 1, err),
                ))
            }
        }
    }

    Ok(())
}
