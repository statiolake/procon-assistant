use super::{Minified, Preprocessed, RawSource, Result};

pub fn preprocess(_quiet: bool, src: RawSource) -> Result<Preprocessed> {
    Ok(Preprocessed(
        src.into_inner()
            .split('\n')
            .map(ToString::to_string)
            .collect(),
    ))
}

pub fn minify(_quiet: bool, preprocessed: Preprocessed) -> Minified {
    Minified(preprocessed.into_inner().join("\n"))
}
