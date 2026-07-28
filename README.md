# porkchop: Visualize Interplanetary Trajectory Constraints

A porkchop plot generator for interplanetary mission design. Given a range of departure and arrival dates, it solves Lambert's problem across a grid of transfer trajectories and visualizes the results as a contour plot of characteristic energies ($C_3$) derived from spacecraft excess velocities ($V_\infty{}$). The brute-force Lambert solver is written in Rust for speed, with Python handling visualization.

---

## Example: 2026 Earth–Mars Transfer Window

The plots below compare Porkchop's output against the reference contour plot from Burke et al. (2010) for the same transfer opportunity — a strong validation of the solver's accuracy.

<img src="https://github.com/mihirb-6/porkchop/blob/main/docs/porkchop_plot.png" alt="Porkchop output" width="50%"/><img src="https://github.com/mihirb-6/porkchop/blob/main/docs/Burke-et-al-comparison.png" alt="Burke et al. 2010 reference" width="50%"/>

> Burke, L. M. *Interplanetary Mission Design Handbook: Earth-to-Mars Mission Opportunities 2026 to 2045.* NASA Technical Reports Server, [ntrs.nasa.gov/citations/20100037210](https://ntrs.nasa.gov/citations/20100037210).

## Example: 2003 Mars *Opportunity* Rover & 2020 Mars *Perseverance* Rover Transfer

<img src="https://github.com/mihirb-6/porkchop/blob/main/docs/opportunity_porkchop_plot.png" alt="Porkchop output" width="50%" height="50%"/><img src="https://github.com/mihirb-6/porkchop/blob/main/docs/perseverance_porkchop_plot.png" alt="Porkchop output" width="50%" height="50%"/>
> The green contours represent the spacecraft's outbound launch asymptote declination, another constraint used in preliminary trajectory design.
---
## Core Features
- Departure/Launch C3 Energy ($\frac{\text{km}^2}{\text{s}^2}$) Contour Plot
- Arrival C3 Energy ($\frac{\text{km}^2}{\text{s}^2}$) Contour Plot
- Declination of Launch Asymptote Plot (deg)
- Right Ascension of Launch Asymptote Plot (deg)
- Lines of Constant Time of Flight (days)
- Manual levels adjustment
---
## How It Works

1. **Rust solver** — iterates over a grid of departure date and time-of-flight pairs and solves Lambert's problem for each, computing the launch and arrival $V_\infty{}$ before computing characteristic energy $C_3$.
2. **Python plotter** — reads the solver's output and renders contour plots with labeled $C_3$ curves.
3. **`config.toml`** - a file to configure everything from the constraints of the launch window to the elements that are displayed on the porkchop plot.
4. **`run.sh`** — runs both the solver and plotter, leaving a porkchop plot in `docs`.

---

## Limitations
1. Lambert's problem assumes ideal two-body (Keplarian) motion, and errors in small perturbations, planetary oblateness, atmospheric drag, solar radiation pressure are not accounted for. (Possible "fix" for this could be applying mid-course corrections).
2. The Rust crate `satkit` only provides JPL ephemeris for the main 8 planets and Pluto, so small body data is not currently implemented.
3. **Ballistic-only trajectories:** we are assuming that the calculated delta-v is delivered instantaneously
4. **No gravity assists**: it's a little silly to plot a porkchop plot from Earth to the Jovian Planets (for example, Europa Clipper used a Mars gravity assist on it's way to Europa)

## Prerequisites

- **Rust** ≥ 1.86: [rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
- **Python** ≥ 3.9: [python.org/downloads](https://www.python.org/downloads)

---

## Installation

```bash
# 1. Clone the repository
git clone https://github.com/mihirb-6/porkchop.git
cd porkchop

# 2. Set up the Python virtual environment
cd plotter-python
python -m venv .venv
source .venv/bin/activate       # On Windows: .venv\Scripts\activate
pip install -r requirements.txt
cd ..
```

---

## Usage

```bash
# Make the runner executable (first time only)
chmod +x run.sh

# Run the full pipeline: Rust solver → Python plotter
./run.sh
```

The plot will be saved to `docs/porkchop_plot.png`.

### Running steps individually

```bash
# Build and run only the Rust solver (outputs raw trajectory data)
cargo build --release
./target/release/porkchop

# Then plot manually (from inside the plotter-python directory, with venv active)
python plot.py
```

> **Tip:** Use `cargo build --release` for significantly faster solve times compared to the debug build.

---

## Project Structure

```
porkchop/
├── solver-rust/        # Lambert solver (Rust) [main.rs, trajectory.rs, elements.rs]
├── plotter-python/     # Contour plot generator (Python) [plot.py]
├── docs/               # Output images and reference plots
├── run.sh              # End-to-end pipeline script
├── config.toml         # TOML to configure search/plot parameters from both Python and Rust
└── README.md
```

---

## License

MIT — see [LICENSE](LICENSE).
