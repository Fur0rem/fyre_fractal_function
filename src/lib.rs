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
    while x <= end {
        let result = f(x, a, b, sum_nb_terms);
        writeln!(writer, "f({}) = {}", x, result)?;
        x += increment;
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
