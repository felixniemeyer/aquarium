# aquarium/scanner

this program generates a color and a normal texture for a fish you've painted and photographed. 
It will remove the background color and crop the image automatically. 

run: `cargo run -- <image_file.png>`

output will be written to `<image_file.png>_[normals.png|colors.png]`

## release
It's much faster if you build it for release: 
`cargo build --release`
and then 
`<release_build_dir>/scanner <image_file.png>

## background color
You can specify the background-color by passing 3 additional parameters after the filename:

`scanner example-fish-image-with-red-background.jpg 255 0 0`
r, g, b, each an integer in the range [0,255]

When cropping doesn't seem to work as expected, make sure you set the bg color accurately and make sure there are no small particles of different color somewhere on the background.