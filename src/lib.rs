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
    let mut x = start;
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

    // while x <= end {
    //     let mut result = 0.0;
    //     for i in 0..=total_non_nans_terms {
    //         let new_term = a_pows[i as usize] * (b_pows[i as usize] * x).cos();
    //         result += new_term;
    //     }
    //     writeln!(writer, "f({}) = {}", x, result)?;
    //     x += increment;
    // }

    let nb_computations = ((end - start) / increment).ceil() as usize;
    let mut x_values = Vec::with_capacity(nb_computations);
    let mut results = Vec::with_capacity(nb_computations);
    const PAR_CHUNK_SIZE: usize = 10000; // Adjust this based on your system's capabilities
    for chunk in (0..=(nb_computations - 1)).step_by(PAR_CHUNK_SIZE) {
        let end_chunk = std::cmp::min(chunk + PAR_CHUNK_SIZE, nb_computations);
        for i in chunk..end_chunk {
            x_values.push(start + i as f64 * increment);
        }
        let chunk_results: Vec<f64> = x_values
            .par_iter()
            .map(|&x| {
                let mut result = 0.0;
                for i in 0..=total_non_nans_terms {
                    let new_term = a_pows[i] * (b_pows[i] * x).cos();
                    result += new_term;
                }
                result
            })
            .collect();
        results.extend(chunk_results);
    }

    for (x, result) in x_values.iter().zip(results.iter()) {
        writeln!(writer, "f({}) = {}", x, result)?;
    }

    // Flush the writer to ensure all data is written to the file
    writer
        .flush()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

pub fn plot_results(result_file: &str, output_file: &str) -> std::io::Result<()> {
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

    let mut fg = Figure::new();
    fg.axes2d().lines(
        &x,
        &y,
        &[Caption("f(x)"), Color(gnuplot::RGBString("black"))],
    );
    fg.save_to_png(output_file, 1920, 1080)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fg.show().unwrap();
    Ok(())
}
