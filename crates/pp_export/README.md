# pp_export

The `export` crate handles rendering of `.ppr` files into PDF / PNG / SVG
outputs, e.g. the things that you'll actually print out and assemble.

All actual rendering logic remains in `pp_draw` - `pp_export` concerns itself
with assembling the file formats.
