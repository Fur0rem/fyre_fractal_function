use std::io::{BufRead, Write};
use std::{f64::consts::PI, fs::File, io::BufWriter};

pub fn f(x: f64, a: f64, b: f64, n: u32) -> f64 {
    let mut sum = 0.0;
    for i in 0..=n {
        let new_term = a.powi(i as i32) * (b.powi(i as i32) * x * PI).cos();
        if new_term.is_nan() {
            eprintln!(
                "Warning: Computation resulted in NaN for x = {}, n = {}! Will return early!",
                x, i
            );
            return sum; // return early
        }
        sum += new_term;
    }
    return sum;
}

use iterator_ilp::IteratorILP;
use rayon::prelude::*;

pub fn compute_all(
    start: f64,
    end: f64,
    increment: f64,
    a: f64,
    b: f64,
    sum_nb_terms: u32,
    writer: &mut BufWriter<File>,
) -> std::io::Result<()> {
    let size = (sum_nb_terms + 1) as usize;
    let mut a_pows = Vec::with_capacity(size);
    let mut b_pows = Vec::with_capacity(size);
    let mut total_non_nans_terms = sum_nb_terms as usize;
    a_pows.push(1.0); // a^0 = 1
    b_pows.push(PI); // b^0 * PI = 1 *
    for i in 1..size {
        let a_im1 = *a_pows.last().unwrap();
        let b_im1 = *b_pows.last().unwrap();
        a_pows.push(a_im1 * a);
        b_pows.push(b_im1 * b);
        if (end * b_pows[i]).is_infinite() {
            eprintln!(
                "Warning: Computation resulted in infinite value for b^{}! Will do only {} terms.",
                i,
                i - 1
            );
            total_non_nans_terms = i - 1;
            break;
        }
    }

    let nb_computations = ((end - start) / increment).ceil() as usize;
    const PAR_CHUNK_SIZE: usize = 100000;
    for is in (0..nb_computations).step_by(PAR_CHUNK_SIZE) {
        let chunk_end = (is + PAR_CHUNK_SIZE).min(nb_computations);
        let chunk: Vec<usize> = (is..chunk_end).collect();
        let results: Vec<f64> = chunk
            .par_iter()
            .map(|&i| {
                let x = start + i as f64 * increment;

                let result = (0..total_non_nans_terms).into_iter().fold_ilp::<4, f64>(
                    || 0.0,
                    |acc, j| acc + a_pows[j] * (b_pows[j] * x).cos(),
                    |acc1, acc2| acc1 + acc2,
                );

                result
            })
            .collect();
        for (i, result) in chunk.iter().zip(results.iter()) {
            writeln!(writer, "f({}) = {}", start + *i as f64 * increment, result)?;
        }
    }

    writer
        .flush()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn plot_results(
    result_file: &str,
    output_file: &str,
    nb_groups: Option<usize>,
) -> std::io::Result<()> {
    use gnuplot::{Caption, Color, Figure};

    let file = File::open(result_file)?;
    let mut file = std::io::BufReader::new(file);
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut line = String::new();
    while file.read_line(&mut line)? > 0 {
        // Format: f(x) = y
        // remove first 2 characters and split by '='
        let linee = line.strip_prefix("f(").unwrap();
        let end_of_paranthesis = linee.find(')').unwrap();
        let x_str = &linee[..end_of_paranthesis];
        let y_str = linee[end_of_paranthesis + 3..].trim(); // Skip " = "
        if let (Ok(x_val), Ok(y_val)) = (x_str.parse::<f64>(), y_str.parse::<f64>()) {
            x.push(x_val);
            y.push(y_val);
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Invalid data in line: '{}'. Expected format: 'f(x) = y'",
                    linee
                ),
            ));
        }
        line.clear();
    }
    if x.is_empty() || y.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No valid data found in the result file",
        ));
    }
    if x.len() != y.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Mismatched lengths of x and y values",
        ));
    }
    if x.len() < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Not enough data points to plot",
        ));
    }

    {
        let mut fg = Figure::new();

        fg.axes2d().lines(
            &x,
            &y,
            &[Caption("f(x)"), Color(gnuplot::RGBString("black"))],
        );
        fg.save_to_png(format!("{}.png", output_file), 1920, 1080)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fg.show().unwrap();

        for i in 1..x.len() {
            y[i] = (y[i] - y[0]) / (x[i] - x[0]); // slope
        }
    }

    let mut fg = Figure::new();

    fg.axes2d().lines(
        &x,
        &y,
        &[
            Caption("Slope f(0) <-> f(x)"),
            Color(gnuplot::RGBString("black")),
        ],
    );
    fg.save_to_png(format!("{}_slope.png", output_file), 1920, 1080)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fg.show().unwrap();

    match nb_groups {
        Some(nb) => {
            let mut counts = vec![0usize; nb + 1];
            let min = &y.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = &y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let range = max - min;
            let increment = range / nb as f64;
            for value in &y {
                let index = ((value - min) / increment).floor() as usize;
                counts[index] += 1;
            }
            let mut groups = Vec::with_capacity(nb + 1);
            for i in 0..=nb {
                groups.push(min + i as f64 * increment);
            }

            let number_of_points = y.len();
            let probas = counts
                .iter()
                .map(|&c| c as f64 / number_of_points as f64)
                .collect::<Vec<_>>();

            let mut fg = Figure::new();
            fg.axes2d().boxes(
                &groups,
                &probas,
                &[Caption("Probability"), Color(gnuplot::RGBString("blue"))],
            );
            fg.save_to_png(&format!("{}_probability_plot.png", output_file), 1920, 1080)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            fg.show().unwrap();
            println!(
                "Probabilities plot saved to {}_probability_plot.png",
                output_file
            );

            // Write counts to a file
            let counts_file = format!("{}_counts.txt", output_file);
            let mut counts_writer = File::create(&counts_file).unwrap();
            for (i, count) in counts.iter().enumerate() {
                writeln!(
                    counts_writer,
                    "({}, {}) : {}",
                    min + i as f64 * increment,
                    min + (i + 1) as f64 * increment,
                    count
                )
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            }
            println!("Counts saved to {}", counts_file);
        }
        None => {
            println!("No grouping specified, skipping counts plot.");
        }
    }
    Ok(())
}
