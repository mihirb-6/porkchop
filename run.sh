cd ~/projects/porkchop/solver-rust/target/release/

#FILE=porkchop

#if [ -e "$FILE" ]; then
#    echo "File exists and is executable."


#else
#    echo "File not found"
#fi

cd ..
cd ..
cargo run

cd ~/projects/porkchop/plotter-python
source .venv/bin/activate
python plot.py
