# gcode-rs

[![Build Status](https://travis-ci.org/Michael-F-Bryan/gcode-rs.svg?branch=master)](https://travis-ci.org/Michael-F-Bryan/gcode-rs)
[![Build status](https://ci.appveyor.com/api/projects/status/1b9pank3tu0oaoy7?svg=true)](https://ci.appveyor.com/project/Michael-F-Bryan/gcode-rs)


A gcode parser designed to turn a stream of characters into valid gcode
instructions.

Currently requires nightly to compile because we use `f32::powi()` which relies
on the unstable `core::num::Float` trait.
