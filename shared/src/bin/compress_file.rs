use shared::compress;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files: Vec<_> = std::env::args().skip(1).collect();

    let in_file = std::fs::read_to_string(&files[0])?;

    let archive = compress::Compress::new();

    let archive = in_file.lines().fold(archive, |mut archive, line| {
        archive.add_record(line);
        archive
    });

    let stored = archive.store_archive();
    std::fs::write(&files[1], &stored)?;

    Ok(())
}
