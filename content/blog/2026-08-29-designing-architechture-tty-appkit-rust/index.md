+++
title = "Designing the Architecture of a TTY with AppKit and Rust"
# description = ""

[taxonomies]
tags = ["rust"] 
+++

At the time of writing, we are nearing the end of August 2026 and Opus 5 has gotten to a point where it _seems_ to be quite good at writing code that _works_ with a good harness; this does not necessarily mean it is _good_ code, and I will always stand by the careful code written by a competent human, as we will always be the arbiters of maintaining worthwhile projects.

Couple days ago I was comparing the dependency count for building a terminal app UI in rust where I hand-rolled one with agentic assistance and compared it with a spinner from the `r3bl` project. `ratatui` pulls in over 300 dependencies and `r3bl` depends on about 88 crates. The simple hand-rolled version had a dep tree that was under 50 crates.

I saw someone joke online how we will start to see adverts played inside our IDEs and terminals and I decided it was time I distract myself by learning into the madness, inspired by [David Plummer's writing a fully native Task Manager](https://x.com/davepl1968/status/2091562472323056070). It was impressive that he could go from a PRD (Product Requirement Document) to a useable piece of software.

> The code is fully native: Direct2D/DirectComposition on Windows, UIKit on Mac, and Qt on Linux. The UI communicates with a shared CPP core, which branches out to platform-specific helpers. There's NO electron, HTML, JSON, etc.

I was inspired by the "shared CPP core" and decided I wanted a Rust core interface through a C FFI and have Opus assist in _teaching_ me through an extensive grill session. My role was to guide all the decisions and the layout of the code base, offering structure and playing to my strengths.

The grilling sessions made me aware how much my ignorance was ignorant of its own ignorance -- some endearingly call these "unknown unknowns".

The project is hosted at https://github.com/bsodmike/experimental-appkit-with-rust.

- Start with the [README](https://github.com/bsodmike/experimental-appkit-with-rust/blob/master/README.md). This covers everything you need to get started on your local machine
  - Running Claude in an isolated docker container
  - `just` commands to run all the tests to build & run `crustty`.
- blog articles generated based on the PRD & ADRs: https://github.com/bsodmike/experimental-appkit-with-rust/tree/master/blog-articles.
- `./docs`: PRDs & ADRs live here.

On the Mac side, here I'm running `just smoke` to test the C boundary

<img src="/images/2026-08-29-designing-architechture-tty-appkit-rust/image-01-smoke.png"/>

Running `just build` to build the AppKit frontend with XCode

<img src="/images/2026-08-29-designing-architechture-tty-appkit-rust/image-02-build.png"/>

This is where it gets interesting. I've now run `Crustty` and I am _building_ it inside the terminal!

<img src="/images/2026-08-29-designing-architechture-tty-appkit-rust/image-03-crustty-build.png"/>

Running `nvim`

<img src="/images/2026-08-29-designing-architechture-tty-appkit-rust/image-04-crustty-nvim.png"/>

## Concluding thoughts

Anyone diving into the Github repository published should look at it more as a _learning experiment_, where we go through fundamentals derived from grilling sessions. I have assisted in detailing the PRD well enough to cover the rust core & FFI boundary, with the agent handling the waist and AppKit side of it.

**Caveat**: the AppKit code may not be the most _ideal_ for maintainability, but this is an aspect that can be refactored in a secondary sprint.

The value I currently see with agentic models and how fast they are improving

- learn on compilers and tools to _learn_ without being mislead.
- rapidly go from _nothing_ to a binary that compiles and runs; it may not be error free, such as runtime panics etc.
- PRDs & ARDs aid with handovers between humans and agents alike -- but they need to be maintained as well.
