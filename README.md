# aquarium

## presentation video
[![watch a presentation](https://img.youtube.com/vi/5SW9_pk5zME/maxresdefault.jpg)](https://youtu.be/5SW9_pk5zME)

## build & run

`cd ./sea`

`cargo run`

(try restarting if it crashes with "unsupoorted dimensions")

## creating fish skins from photos

`cd ./scanner`

edit `./src/lib.rs` to use the right photo file. 
If necessary adjust BG_COLOR and COL_DISTANCE_SQUARED constants to match the "green screen" color in the photo. 

`run cargo test --release` to execute.

copy `test_colors.png` & `test_normals.png` over to `../sea/fish` and refer to them in `../sea/src/main.rs`.
