use std::{
    env,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::Path,
};

use hackvm::parse;

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    assert!(args.len() >= 2, "Usage: hackvm <filename>.vm");
    let infile = Path::new(&args[1]);
    // let infile = Path::new("hello.vm");
    assert!(
        infile.extension().and_then(|ext| ext.to_str()) == Some("vm"),
        "Expected .vm file"
    );
    let outpath = infile.with_extension("asm");
    println!("{} | {}", infile.display(), outpath.display());

    let content = fs::read_to_string(infile)?;
    let outfile = File::create(outpath)?;
    let mut writer = BufWriter::new(outfile);
    let filestem = infile.file_stem().and_then(|stem| stem.to_str()).unwrap();

    for (n, line) in content.lines().enumerate() {
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        // println!("{}", line);

        match parse(line) {
            Ok(command) => {
                dbg!(&command);
                let asm = command.to_asm(filestem);
                println!("{asm}");
                writeln!(writer, "{}", asm)?;
            }
            Err(err) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Error at line {}: {}", n + 1, err),
                ))
            }
        }
    }

    writer.flush()?;
    Ok(())
}
