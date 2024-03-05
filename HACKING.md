# Conventions

Components are named so that they are something the entity HAS rather than
something the entity IS, e.g `Parent` is a component you would expect a *child*
in a hierarchy to have that references the parents.

# Architecture

Secondary crates in the workspace define generalised libraries that don't
depend on the main code base. Some of these define bevy plugins but other are
independent of bevy.

The main crate is split into several modules that typically exports a bevy
plugin and related components for the rest of the game to use. Large module
files are chopped up and put into a directory with `mod.rs` that exports the
public interface.
