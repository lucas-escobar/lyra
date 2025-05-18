# Lyra

Lyra is a framework for creating music. The goal of this project is to have
all of the necessary tools for music creation in one place. This includes
the tools for writing music notation, rendering audio, and mixing/mastering
tracks.

Lyra does not contain any GUI, it is intended to enable programmers to write
music with code.

## Architecture

There are three main components to this project. It is possible for each layer
to be used on its own.

### Composition

This is where the score is defined. A score contains parts, measures, notes and
more on based on western music notation. Specifically, this layer is designed
to be compatable with the [MusicXML specification](https://www.musicxml.com/).

### Audio Rendering

Score parts defined in the composition layer can be consumed by this layer to
produce audio buffers. The buffers produced by this layer are the most raw
audio representation of the part. This is where instruments are modelled.

Currently supports rendering from a part but it is intended to be able to render
based on other instructions (for ambient soundscapes without musical context
for example).

### Digital Signal Processing

The audio buffers produced by the rendering layer are placed into tracks.
Effect chains, automation, panning, etc. can be applied to these tracks.
Multiple tracks are then combined into a single output.
