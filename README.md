# Rust bindings for GRnvS's libraw library

Simply create a rust project using `cargo init . --name <assignment-name>` and copy the Makefile into the project folder.
Finally, copy `grnvs.rs` into `src/`.

In order to use the Makefile, adjust the `CRATE_NAME` variable in the first line.
Before building locally, run `export ONLINE=1` in your shell to signal to make that you are running online.
Before commiting, run `make common/cargo_deps.tar.gz` and commit `common/` and `.cargo`, too.
This will package all your crates and ship them to the tester.
Remove both directorys before continuing to work locally, cargo will fail otherwise.

:warning: **Important:** You will have to cite this repository.
Failure to do so will likely result in an accusation of plagiarism!
Don't remove the copyright notice in any file!
