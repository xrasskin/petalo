// ----------------------------------- CLI -----------------------------------
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "mlem", about = "TODO: describe what this does")]
pub struct Cli {

    /// Number of MLEM iterations to perform
    #[structopt(short, long, default_value = "5")]
    iterations: usize,

    /// Voxel box half-widths
    #[structopt(short, long, parse(try_from_str = parse_triplet::<F>), default_value = "180,180,180")]
    pub size: (F, F, F),

    /// Number of voxels in each dimension
    #[structopt(short, long, parse(try_from_str = parse_triplet::<usize>), default_value = "60,60,60")]
    pub n_voxels: (usize, usize, usize),

}

fn parse_triplet<T: std::str::FromStr>(s: &str) -> Result<(T,T,T), <T as std::str::FromStr>::Err> {
    let v = s.split(",").collect::<Vec<_>>();
    assert!(v.len() == 3);
    let x = v[0].parse()?;
    let y = v[1].parse()?;
    let z = v[2].parse()?;
    Ok((x, y, z))
}

// --------------------------------------------------------------------------------

use std::error::Error;

use ndarray::prelude::*;

use petalo::weights as pet;

type F = pet::Length;


fn main() -> Result<(), Box<dyn Error>> {

    let args = Cli::from_args();

    println!("Float precision: {} bits", petalo::weights::PRECISION);

    // Set up progress reporting and timing
    use std::time::Instant;
    let mut now = Instant::now();

    let mut report_time = |message| {
        println!("{}: {} ms", message, now.elapsed().as_millis());
        now = Instant::now();
    };

    // If LOR data file not present, download it.
    let filename = "run_fastfastmc_1M_events.bin32";
    if !std::path::Path::new(&filename).exists() {
        println!("Fetching data file containing LORs: It will be saved as '{}'.", filename);
        let wget = std::process::Command::new("wget")
            .args(&["https://gateway.pinata.cloud/ipfs/QmQN54iQZWtkcJ24T3NHZ78d9dKkpbymdpP5sfvHo628S2/run_fastfastmc_1M_events.bin32"])
            .output()?;
        println!("wget status: {}", wget.status);
        report_time("Downloaded LOR data");
    }

    // Read event data from disk into memory
    let measured_lors = petalo::io::read_lors(filename)?;
    report_time("Loaded LOR data from local disk");

    // Define extent and granularity of voxels
    let vbox;
    {
        let size = (args.size.0 / 2.0, args.size.1 / 2.0, args.size.2 / 2.0);
        vbox = pet::VoxelBox::new(size, args.n_voxels);
    }
    // TODO: sensitivity matrix, all ones for now
    let sensitivity_matrix = pet::Image::ones(vbox).data;
    // TODO: noise
    let noise = pet::Noise;

    // Perform MLEM iterations
    for (n, image) in (pet::Image::mlem(vbox, &measured_lors, &sensitivity_matrix, &noise))
        .take(args.iterations)
        .enumerate() {
            report_time("iteration");
            let data: ndarray::Array3<F> = image.data;
            let path = std::path::PathBuf::from(format!("raw_data/deleteme{:03}.raw", n));
            petalo::io::write_bin(data.iter(), &path)?;
            report_time("Wrote raw bin");
            // TODO: step_by for print every
        }

    Ok(())
}
