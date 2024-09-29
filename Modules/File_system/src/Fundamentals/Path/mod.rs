/// Path module of the File_system library.
/// 
/// This module contains the different types that represent paths.
/// 
/// A path is a string that represents the location of a file or directory in a file system.
/// It is composed of components separated by a separator character.
/// A file path should start with a [Separator] and not end with a [Separator].
mod Components;
mod Path_owned;
mod Path_reference;

pub use Components::*;
pub use Path_owned::*;
pub use Path_reference::*;

/// Separator character used in paths.
pub const Separator: char = '/';

/// Extension separator character used in paths.
pub const Extension_separator: char = '.';
