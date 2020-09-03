# Islands
WIP of a procedural island generator.

As it stands now it generates a height map of a single island and outputs it into terrain.png

# Build and Run
As the project is Rust-based, there needs to be a functioning Rust compiler installed.

To compile and run, use `cargo run --release`. 
The release flag is not strictly needed, but it speeds up the program a lot and doesn't take much longer to compile.

# Examples
![Heightmap Image](meta/example-heightmap.png)

# Performance
Running on an i5-7600k it takes about 0.28 seconds for an 800 by 800 terrain.

The time it takes to complete scales roughly with `O(W*H*log(W*H))`.

```
$ cargo run --release
   Compiling islands v0.1.0
    Finished release [optimized] target(s) in 0.89s
     Running `target/release/islands`

-- Island Generator --

Generating a 800x800 island.
Generating Simplex Map	(0.084 s)
Finding Flow Targets	(0.032 s)
Generating River Map	(0.020 s)
Generating Lake Map	(0.011 s)
Adjusting River Map	(0.004 s)
Generating Terrain	(0.107 s)
Exporting Heightmap	(0.008 s)
Exporting River/Lake	(0.015 s)
Total			[0.281 s]

```
