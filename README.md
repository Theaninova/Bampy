# Bampy

[Discord](https://discord.gg/v6qBb76zkK), any help is welcome.

A work-in-progress slicer universal non-planar slicing, currently in proof-of-concept state.

The slicer was written from scratch, using Rust and WASM to run in the browser.

## Explanation

Non-planar slicing is nothing new, however so far what you see everywhere are either just hardware projects or hand-crafted gcode.
Full Control was an interesting step, but ultimately not something everyone can use.
Same with bending gcode, conical slicing - they require tons of manual work and pre-thinking and an object designed for it.

The challenge with a non-planar slicer is that the toolhead would bump into already printed objects while printing the non-planar surfaces.
In this first step I managed to create a slicer that finds toolpaths that work around this issue.

The next step is getting the slicer to a point where it can output gcode following the prepared toolpaths and adding infill, as well as fixing the bugs that would cause the toolhead to print paths under existing layers (you can see it at the front of the ship).
