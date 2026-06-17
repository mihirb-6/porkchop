# Porkchop - A Tool to Visualize Interplanetary Trajectories
---

## Example Porkchop Plot of the 2026 Earth-Mars Transfer Opportunity
### Porkchop's output (left) vs contour plot (right) found in Burke et al. (2010)

<img src="https://github.com/mihirb-6/porkchop/blob/main/docs/porkchop_plot.png" alt="porkchop" width="60%"/><img src="https://github.com/mihirb-6/porkchop/blob/main/docs/Burke-et-al-comparison.png" alt="comparison" width="40%"/>
Source: Burke, Laura M. “Interplanetary Mission Design Handbook: Earth-to-Mars Mission Opportunities 2026 to 2045 - NASA Technical Reports Server (NTRS).” NASA, NASA, ntrs.nasa.gov/citations/20100037210.

## Installation

- Install the latest version of Rust (>=1.96): https://rust-lang.org/tools/install/
- Install python: https://www.python.org/downloads/
- Clone the repo
  ```bash
  git clone https://github.com/mihirb-6/porkchop.git
  cd porkchop
  ```
- Configure a python virtual environment:
  ```bash
  cd ~/porkchop/plotter-python
  python -m venv .venv
  ```
- Use `source .venv/bin/activate` to activate the venv
- Install the required dependencies using `pip install -r requirements.txt`

## Usage
```bash
# create an executable of run.sh
chmod +x run.sh
# run the solver first in Rust, then plot with Python
./run.sh

# (alternative)
cargo build --release # for faster runtime speed
./target/release/porkchop # will only run the solver
```
