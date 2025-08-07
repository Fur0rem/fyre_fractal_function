use clap::Parser;
use fyre_fractal_function::{compute_all, plot_results};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Number of terms in the sum
    #[arg(short = 'n', long)]
    sum_nb_terms: u32,

    /// Start of the interval
    #[arg(short = 's', long)]
    start: f64,

    /// End of the interval
    #[arg(short = 'e', long)]
    end: f64,

    /// The a coefficient
    #[arg(short, long)]
    a: f64,

    /// The b coefficient
    #[arg(short, long)]
    b: f64,

    /// Output directory of the results
    #[arg(short, long, default_value = "results")]
    output_dir: String,

    /// The increment to use for the computations
    #[arg(short, long)]
    increment: Option<f64>,
}

fn main() {
    // Get args
    let args = Args::parse();
    let increment = args.increment.unwrap_or(args.end.next_up() - args.end);

    // Check args
    if increment <= 0.0 {
        panic!("Increment must be a positive number");
    }
    if increment > args.end - args.start {
        panic!("Increment must be less than the range from start to end");
    }
    if args.start >= args.end {
        panic!("Start must be less than end");
    }
    if args.sum_nb_terms == 0 {
        panic!("Number of terms in the sum must be greater than 0");
    }

    // Print info to user
    println!(
        "Will do {} computations by increments of {} from {} to {}",
        (args.end - args.start) / increment,
        increment,
        args.start,
        args.end
    );
    println!(
        "Using a = {}, b = {}, and sum_nb_terms = {}",
        args.a, args.b, args.sum_nb_terms
    );
    println!("Results will be saved in {}", args.output_dir);

    // Open the output directory and output file
    std::fs::create_dir_all(&args.output_dir).expect("Failed to create output directory");
    let output_files = format!(
        "{}/results_start_{}_end_{}_a_{}_b_{}_n_{}",
        args.output_dir, args.start, args.end, args.a, args.b, args.sum_nb_terms
    );
    let output_file_path = format!("{}.txt", output_files);
    let path = std::path::Path::new(&output_file_path);
    if path.exists() {
        std::fs::remove_file(&output_file_path).expect("Failed to remove existing output file");
    } else {
        println!("Output file does not exist, creating a new one.");
    }
    let output_file =
        std::fs::File::create(&output_file_path).expect("Failed to create output file");
    let mut writer = std::io::BufWriter::new(output_file);

    compute_all(
        args.start,
        args.end,
        increment,
        args.a,
        args.b,
        args.sum_nb_terms,
        &mut writer,
    )
    .unwrap();

    println!("Results saved to {}", output_file_path);
    println!("Done!");

    plot_results(&output_file_path, &format!("{}.png", output_files)).unwrap();
}
