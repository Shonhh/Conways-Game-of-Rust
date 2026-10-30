//! Reproducible experiments using the same engine as the terminal application.
use crate::grid::{CellState, Grid};
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};
use std::time::{Duration, Instant};

const HELP: &str = "Conway's Game of Rust

Usage:
  conway_game_of_rust                         Launch the terminal interface
  conway_game_of_rust --headless [OPTIONS]    Run without a terminal

Options:
  --width N           Grid width (default: 128; positive)
  --height N          Grid height (default: 80; positive)
  --generations N     Number of updates (default: 100; zero is valid)
  --pattern NAME      Centered empty, block, blinker, or glider (default: glider)
  --seed N            Initialize randomly with this u64 seed instead of a pattern
  --density P         Random live-cell probability, 0 through 1 (default: 0.3)
  --output PATH       Save CSV to a new file (default: stdout; '-' also means stdout)
  --help, -h          Show this help

Boundaries are fixed, with dead cells outside the grid. CSV includes generation 0.
Elapsed time is cumulative update time only, excluding initialization, statistics,
checksums, and CSV I/O. Output files are never overwritten.
";

#[derive(Debug)]
struct Config {
    width: usize,
    height: usize,
    generations: u64,
    pattern: String,
    seed: Option<u64>,
    density: f64,
    output: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 128,
            height: 80,
            generations: 100,
            pattern: "glider".into(),
            seed: None,
            density: 0.3,
            output: None,
        }
    }
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn parse(args: &[String]) -> io::Result<Config> {
    let mut config = Config::default();
    let mut headless = false;
    let mut pattern_set = false;
    let mut density_set = false;
    let mut seen = std::collections::HashSet::new();
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        if !seen.insert(flag.as_str()) {
            return Err(invalid(format!("duplicate option: {flag}")));
        }
        if flag == "--headless" {
            headless = true;
            continue;
        }
        if !matches!(
            flag.as_str(),
            "--width"
                | "--height"
                | "--generations"
                | "--pattern"
                | "--seed"
                | "--density"
                | "--output"
        ) {
            return Err(invalid(format!("unknown option: {flag}; use --help")));
        }

        let value = args
            .next()
            .ok_or_else(|| invalid(format!("missing value for {flag}")))?;
        let bad_value = || invalid(format!("invalid value for {flag}: {value}"));
        match flag.as_str() {
            "--width" => config.width = value.parse().map_err(|_| bad_value())?,
            "--height" => config.height = value.parse().map_err(|_| bad_value())?,
            "--generations" => config.generations = value.parse().map_err(|_| bad_value())?,
            "--pattern" => {
                config.pattern = value.clone();
                pattern_set = true;
            }
            "--seed" => config.seed = Some(value.parse().map_err(|_| bad_value())?),
            "--density" => {
                config.density = value.parse().map_err(|_| bad_value())?;
                density_set = true;
            }
            "--output" => config.output = Some(value.clone()),
            _ => unreachable!(),
        }
    }

    if !headless {
        return Err(invalid("options require --headless; use --help"));
    }
    if config.width == 0 || config.height == 0 {
        return Err(invalid("grid dimensions must be positive"));
    }
    if config
        .width
        .checked_mul(config.height)
        .filter(|&size| size <= isize::MAX as usize / std::mem::size_of::<CellState>())
        .is_none()
    {
        return Err(invalid("grid dimensions exceed addressable storage"));
    }
    if !config.density.is_finite() || !(0.0..=1.0).contains(&config.density) {
        return Err(invalid("density must be a finite number between 0 and 1"));
    }
    if pattern_set && config.seed.is_some() {
        return Err(invalid("choose either --pattern or --seed"));
    }
    if density_set && config.seed.is_none() {
        return Err(invalid("--density requires --seed"));
    }

    // Validate before allocating or opening an output file.
    if config.seed.is_none() {
        let (width, height, _) = pattern(&config.pattern)?;
        if width > config.width || height > config.height {
            return Err(invalid("pattern does not fit in the requested grid"));
        }
    }
    Ok(config)
}

type Pattern = (usize, usize, &'static [(usize, usize)]);

fn pattern(name: &str) -> io::Result<Pattern> {
    match name {
        "empty" => Ok((0, 0, &[])),
        "block" => Ok((2, 2, &[(0, 0), (0, 1), (1, 0), (1, 1)])),
        "blinker" => Ok((3, 1, &[(0, 0), (0, 1), (0, 2)])),
        "glider" => Ok((3, 3, &[(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)])),
        _ => Err(invalid(format!("unknown pattern: {name}"))),
    }
}

/// SplitMix64, with explicit wrapping arithmetic for platform-independent seeds.
fn next_random(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

fn initialize(config: &Config) -> io::Result<Grid> {
    let mut grid = Grid::new(config.width, config.height);

    if let Some(mut seed) = config.seed {
        for row in 0..config.height {
            for col in 0..config.width {
                // Use the upper 53 bits to produce a value in [0, 1).
                let random = (next_random(&mut seed) >> 11) as f64 / 9007199254740992.0;
                if random < config.density {
                    grid.set(row, col, CellState::Alive);
                }
            }
        }
    } else {
        let (width, height, cells) = pattern(&config.pattern)?;
        let top = (config.height - height) / 2;
        let left = (config.width - width) / 2;
        for &(row, col) in cells {
            grid.set(top + row, left + col, CellState::Alive);
        }
    }

    Ok(grid)
}

fn write_row(
    config: &Config,
    grid: &Grid,
    generation: u64,
    elapsed: Duration,
    output: &mut impl Write,
) -> io::Result<()> {
    let initialization = if config.seed.is_some() {
        "random"
    } else {
        &config.pattern
    };

    let seed = config.seed.map(|seed| seed.to_string()).unwrap_or_default();
    let density = if config.seed.is_some() {
        config.density.to_string()
    } else {
        String::new()
    };

    writeln!(
        output,
        "{},{},fixed-dead,{},{},{},{},{},{},{:016x}",
        config.width,
        config.height,
        initialization,
        seed,
        density,
        generation,
        elapsed.as_nanos(),
        grid.live_count(),
        grid.checksum()
    )
}

fn run(config: &Config, output: &mut impl Write) -> io::Result<()> {
    let mut grid = initialize(config)?;
    writeln!(output, "width,height,boundary,initialization,seed,density,generation,elapsed_update_ns,live_cells,checksum")?;

    let mut elapsed = Duration::ZERO;
    write_row(config, &grid, 0, elapsed, output)?;

    for generation in 1..=config.generations {
        let start = Instant::now();
        grid.next_generation();
        elapsed += start.elapsed();
        write_row(config, &grid, generation, elapsed, output)?;
    }

    output.flush()
}

/// Dispatch arguments before Ratatui initializes raw terminal mode.
pub fn run_cli(args: &[String]) -> io::Result<()> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        return io::stdout().lock().write_all(HELP.as_bytes());
    }

    let config = parse(args)?;
    match config.output.as_deref() {
        None | Some("-") => run(&config, &mut BufWriter::new(io::stdout().lock())),
        Some(path) => {
            let file = OpenOptions::new().write(true).create_new(true).open(path)?;
            run(&config, &mut BufWriter::new(file))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(args: &[&str]) -> io::Result<Config> {
        parse(&args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn invalid_arguments_are_rejected() {
        for args in [
            vec!["--width", "5"],
            vec!["--headless", "--width", "0"],
            vec!["--headless", "--height", "-1"],
            vec!["--headless", "--generations", "bad"],
            vec!["--headless", "--generations", "-1"],
            vec!["--headless", "--width"],
            vec!["--headless", "--unknown"],
            vec!["--headless", "--pattern", "unknown"],
            vec!["--headless", "--width", "1"],
            vec!["--headless", "--seed", "1", "--pattern", "block"],
            vec!["--headless", "--density", "0.5"],
            vec!["--headless", "--seed", "1", "--density", "NaN"],
            vec!["--headless", "--seed", "1", "--density", "inf"],
            vec!["--headless", "--seed", "1", "--density", "1.1"],
            vec!["--headless", "--seed", "1", "--density", "-0.1"],
            vec!["--headless", "--seed", "1", "--seed", "2"],
        ] {
            assert!(config(&args).is_err(), "accepted {args:?}");
        }
        assert!(config(&["--headless", "--width", &usize::MAX.to_string()]).is_err());
    }

    #[test]
    fn seeded_runs_are_identical_except_for_timing() {
        let config = config(&[
            "--headless",
            "--width",
            "9",
            "--height",
            "7",
            "--seed",
            "42",
            "--generations",
            "6",
        ])
        .unwrap();
        let mut first = Vec::new();
        let mut second = Vec::new();
        run(&config, &mut first).unwrap();
        run(&config, &mut second).unwrap();
        let without_time = |bytes: Vec<u8>| -> Vec<String> {
            String::from_utf8(bytes)
                .unwrap()
                .lines()
                .map(|line| {
                    line.split(',')
                        .enumerate()
                        .filter(|(index, _)| *index != 7)
                        .map(|(_, field)| field)
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .collect()
        };
        assert_eq!(without_time(first), without_time(second));
    }

    #[test]
    fn random_generator_has_a_known_sequence() {
        let mut seed = 0;
        assert_eq!(next_random(&mut seed), 0xe220a8397b1dcdaf);
        assert_eq!(next_random(&mut seed), 0x6e789e6aa1b965f4);
    }

    #[test]
    fn density_endpoints_and_seed_zero_work() {
        for (density, expected) in [("0", 0), ("1", 12)] {
            let config = config(&[
                "--headless",
                "--width",
                "4",
                "--height",
                "3",
                "--seed",
                "0",
                "--density",
                density,
            ])
            .unwrap();
            assert_eq!(initialize(&config).unwrap().live_count(), expected);
        }
    }

    #[test]
    fn csv_includes_initial_state_and_final_generation() {
        let config = config(&[
            "--headless",
            "--width",
            "5",
            "--height",
            "5",
            "--pattern",
            "blinker",
            "--generations",
            "2",
        ])
        .unwrap();
        let mut output = Vec::new();
        run(&config, &mut output).unwrap();
        let text = String::from_utf8(output).unwrap();
        let rows: Vec<Vec<&str>> = text.lines().map(|line| line.split(',').collect()).collect();
        assert_eq!(rows.len(), 4);
        assert!(rows.iter().all(|row| row.len() == 10));
        assert_eq!(rows[1][6], "0");
        assert_eq!(rows[1][7], "0");
        assert_eq!(rows[3][6], "2");
        assert_eq!(rows[1][9], rows[3][9]);
        assert_ne!(rows[1][9], rows[2][9]);
        assert!(rows[1..].iter().all(|row| row[8] == "3"));
    }

    #[test]
    fn zero_generations_only_writes_initial_state() {
        let config = config(&[
            "--headless",
            "--pattern",
            "empty",
            "--width",
            "1",
            "--height",
            "1",
            "--generations",
            "0",
        ])
        .unwrap();
        let mut output = Vec::new();
        run(&config, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap().lines().count(), 2);
    }

    #[test]
    fn output_errors_propagate() {
        struct FailingWriter;
        impl Write for FailingWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "test"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        assert_eq!(
            run(&Config::default(), &mut FailingWriter)
                .unwrap_err()
                .kind(),
            io::ErrorKind::BrokenPipe
        );
    }
}
